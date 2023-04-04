use async_trait::async_trait;
use futures::StreamExt;
use si_data_nats::NatsClient;
use std::convert::TryInto;
use telemetry::prelude::*;
use tokio::{sync::mpsc, task::JoinSet};

use crate::{
    job::{
        consumer::JobInfo,
        producer::{BlockingJobError, BlockingJobResult, JobProducer, JobProducerError},
        queue::JobQueue,
    },
    DalContext,
};

use super::{JobQueueProcessor, JobQueueProcessorError, JobQueueProcessorResult};

const NATS_JOB_QUEUE: &str = "pinga-jobs";

#[derive(Clone, Debug)]
pub struct NatsProcessor {
    client: NatsClient,
    queue: JobQueue,
    // Drop guard that will ensure the receiver end never returns until all NatsProcessors are gone
    // Necessary as since we spawn a task to enqueue the jobs hyper's graceful shutdown won't wait on us
    // And since we may not have pushed all jobs (or any) `NatsClient::close` will not wait until we have finished
    //
    // We never send anything to this, it's just to add reactivity
    _alive_marker: mpsc::Sender<()>,
}

impl NatsProcessor {
    pub fn new(client: NatsClient, _alive_marker: mpsc::Sender<()>) -> Self {
        Self {
            client,
            _alive_marker,
            queue: JobQueue::new(),
        }
    }

    async fn push_all_jobs(&self) -> JobQueueProcessorResult<()> {
        while let Some(element) = self.queue.fetch_job().await {
            let job_info: JobInfo = element.try_into()?;

            if let Err(err) = self
                .client
                .publish(NATS_JOB_QUEUE, serde_json::to_vec(&job_info)?)
                .await
            {
                error!("Nats job push failed, some jobs will be dropped");
                return Err(JobQueueProcessorError::Transport(Box::new(err)));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl JobQueueProcessor for NatsProcessor {
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>, _ctx: &DalContext) {
        self.queue.enqueue_job(job).await
    }

    async fn block_on_job(&self, job: Box<dyn JobProducer + Send + Sync>) -> BlockingJobResult {
        let job_info: JobInfo = job
            .try_into()
            .map_err(|e: JobProducerError| BlockingJobError::JobProducer(e.to_string()))?;
        let job_reply_inbox = self.client.new_inbox();
        let mut reply_subscription = self
            .client
            .subscribe(&job_reply_inbox)
            .await
            .map_err(|e| BlockingJobError::Nats(e.to_string()))?;
        self.client
            .publish_request(
                NATS_JOB_QUEUE,
                &job_reply_inbox,
                serde_json::to_vec(&job_info)
                    .map_err(|e| BlockingJobError::Serde(e.to_string()))?,
            )
            .await
            .map_err(|e| BlockingJobError::Nats(e.to_string()))?;

        match reply_subscription.next().await {
            Some(Ok(message)) => {
                match serde_json::from_slice::<BlockingJobResult>(message.data())
                    .map_err(|e| BlockingJobError::Serde(e.to_string()))?
                {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err),
                }
            }
            Some(Err(err)) => {
                error!("Internal nats error: {err}");
                Err(BlockingJobError::Nats(err.to_string()))
            }
            None => Err(BlockingJobError::Nats(
                "Subscription or connection no longer valid".to_string(),
            )),
        }
    }

    async fn block_on_jobs(
        &self,
        jobs: Vec<Box<dyn JobProducer + Send + Sync>>,
    ) -> BlockingJobResult {
        let mut dispatched_jobs = JoinSet::new();

        // Fan out, dispatching all queued jobs to pinga over nats.
        for job in jobs {
            let job_processor = self.clone();
            dispatched_jobs.spawn(async move {
                let _ = job_processor.block_on_job(job).await;
            });
        }

        // Wait for all queued jobs to finish (regardless of success), before exiting.
        loop {
            if dispatched_jobs.join_next().await.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        let processor = self.clone();
        tokio::spawn(async move {
            if let Err(err) = processor.push_all_jobs().await {
                error!("Unable to push jobs to nats: {err}");
            }
        });

        Ok(())
    }

    async fn blocking_process_queue(&self) -> JobQueueProcessorResult<()> {
        self.block_on_jobs(self.queue.drain().await).await?;

        Ok(())
    }
}
