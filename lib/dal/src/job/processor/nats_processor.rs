use async_trait::async_trait;
use si_data_nats::NatsClient;
use std::{collections::VecDeque, convert::TryInto};
use telemetry::prelude::*;
use tokio::sync::mpsc;

use crate::{
    job::{
        consumer::{JobInfo, NextJobInfo},
        producer::{JobProducer, JobProducerError},
        queue::JobQueue,
    },
    DalContext,
};

use super::{JobQueueProcessor, JobQueueProcessorError, JobQueueProcessorResult};

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
            let mut job_info: JobInfo = element.job.try_into()?;
            if element.wait_for_execution {
                job_info.subsequent_jobs = self
                    .queue
                    .empty()
                    .await
                    .into_iter()
                    .map(|el| {
                        Ok(NextJobInfo {
                            job: el.job.try_into()?,
                            wait_for_execution: el.wait_for_execution,
                        })
                    })
                    .collect::<Result<VecDeque<_>, JobProducerError>>()?;
            }
            if let Err(err) = self
                .client
                .publish("pinga-jobs", serde_json::to_vec(&job_info)?)
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

    async fn enqueue_blocking_job(
        &self,
        job: Box<dyn JobProducer + Send + Sync>,
        _ctx: &DalContext,
    ) {
        self.queue.enqueue_blocking_job(job).await
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        if let Err(err) = self.push_all_jobs().await {
            error!("Unable to push jobs to nats: {err}");
        }

        Ok(())
    }
}
