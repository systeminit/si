use async_trait::async_trait;
use futures::StreamExt;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use tokio::task::JoinSet;

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
    pinga_subject: String,
}

impl NatsProcessor {
    pub fn new(client: NatsClient) -> Self {
        let pinga_subject = if let Some(prefix) = client.metadata().subject_prefix() {
            format!("{prefix}.{NATS_JOB_QUEUE}")
        } else {
            NATS_JOB_QUEUE.to_owned()
        };

        Self {
            client,
            queue: JobQueue::new(),
            pinga_subject,
        }
    }

    async fn push_all_jobs(&self) -> JobQueueProcessorResult<()> {
        while let Some(element) = self.queue.fetch_job().await {
            let job_info = JobInfo::new(element)?;

            if let Err(err) = self
                .client
                .publish(&self.pinga_subject, serde_json::to_vec(&job_info)?)
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
        let mut job_info = JobInfo::new_blocking(job)
            .map_err(|e: JobProducerError| BlockingJobError::JobProducer(e.to_string()))?;

        job_info.blocking = true;

        let job_reply_inbox = self.client.new_inbox();
        let mut reply_subscription = self
            .client
            .subscribe(&job_reply_inbox)
            .await
            .map_err(|e| BlockingJobError::Nats(e.to_string()))?;
        self.client
            .publish_with_reply(
                &self.pinga_subject,
                &job_reply_inbox,
                serde_json::to_vec(&job_info)
                    .map_err(|e| BlockingJobError::Serde(e.to_string()))?,
            )
            .await
            .map_err(|e| BlockingJobError::Nats(e.to_string()))?;

        match reply_subscription.next().await {
            Some(message) => serde_json::from_slice::<BlockingJobResult>(message.data())
                .map_err(|e| BlockingJobError::Serde(e.to_string()))?,
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
            let job_processor = Self::new(self.client.clone());
            dispatched_jobs.spawn(async move { job_processor.block_on_job(job).await });
        }

        let mut results = Vec::new();
        // Wait for all queued jobs to finish (regardless of success), before exiting.
        loop {
            match dispatched_jobs.join_next().await {
                // All jobs done.
                None => break,
                Some(Ok(Ok(_))) => { /* Nothing to do. Job succeeded. */ }
                Some(Ok(Err(job_error))) => {
                    results.push(job_error);
                }
                Some(Err(join_err)) => {
                    results.push(BlockingJobError::JobExecution(join_err.to_string()));
                }
            }
        }

        if !results.is_empty() {
            Err(BlockingJobError::JobExecution(
                results
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n"),
            ))
        } else {
            Ok(())
        }
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
