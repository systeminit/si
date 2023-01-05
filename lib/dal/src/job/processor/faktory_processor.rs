use async_trait::async_trait;
use faktory_async::Client;
use std::convert::TryInto;
use telemetry::prelude::*;
use tokio::sync::mpsc;

use crate::{
    job::{
        consumer::{JobConsumerCustomPayload, JobConsumerError, JobInfo},
        producer::JobProducer,
        queue::JobQueue,
    },
    DalContext,
};

use super::{JobQueueProcessor, JobQueueProcessorError, JobQueueProcessorResult};

#[derive(Clone, Debug)]
pub struct FaktoryProcessor {
    client: Client,
    queue: JobQueue,
    // Drop guard that will ensure the receiver end never returns until all FaktoryProcessors are gone
    // Necessary as since we spawn a task to enqueue the jobs hyper's graceful shutdown won't wait on us
    // And since we may not have pushed all jobs (or any) `Client::close` will not wait until we have finished
    //
    // We never send anything to this, it's just to add reactivity
    _alive_marker: mpsc::Sender<()>,
}

impl FaktoryProcessor {
    pub fn new(client: Client, _alive_marker: mpsc::Sender<()>) -> Self {
        Self {
            client,
            _alive_marker,
            queue: JobQueue::new(),
        }
    }

    async fn push_all_jobs(&self) -> JobQueueProcessorResult<()> {
        while let Some(job) = self.queue.fetch_job().await {
            let job_info: JobInfo = job.try_into()?;
            if let Err(err) = self.client.push(job_info.into()).await {
                error!("Faktory push failed, some jobs will be dropped");
                return Err(JobQueueProcessorError::Transport(Box::new(err)));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl JobQueueProcessor for FaktoryProcessor {
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>, _ctx: &DalContext) {
        self.queue.enqueue_job(job).await
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        let processor = self.clone();
        tokio::spawn(async move {
            if let Err(err) = processor.push_all_jobs().await {
                error!("Unable to push jobs to faktory: {err}");
            }
        });

        Ok(())
    }
}

impl From<JobInfo> for faktory_async::Job {
    fn from(job: JobInfo) -> Self {
        let mut faktory_job = faktory_async::Job::new(job.kind, job.args);

        faktory_job.retry = job.retry;
        faktory_job.at = job.at;
        faktory_job.custom = job.custom.extra;

        faktory_job
    }
}

impl TryFrom<faktory_async::Job> for JobInfo {
    type Error = JobConsumerError;

    fn try_from(job: faktory_async::Job) -> Result<Self, Self::Error> {
        let custom: JobConsumerCustomPayload =
            serde_json::from_value(serde_json::to_value(job.custom.clone())?)?;

        Ok(JobInfo {
            id: job.id().to_string(),
            kind: job.kind().to_string(),
            queue: job.queue.clone(),
            created_at: job.created_at,
            enqueued_at: job.enqueued_at,
            at: job.at,
            args: job.args().to_vec(),
            retry: job.retry,
            custom,
        })
    }
}
