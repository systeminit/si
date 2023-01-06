use async_trait::async_trait;
use si_data_nats::NatsClient;
use std::convert::TryInto;
use telemetry::prelude::*;
use tokio::sync::mpsc;

use crate::{
    job::{consumer::JobInfo, producer::JobProducer, queue::JobQueue},
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
        while let Some(job) = self.queue.fetch_job().await {
            let job_info: JobInfo = job.try_into()?;
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

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        let processor = self.clone();
        tokio::spawn(async move {
            if let Err(err) = processor.push_all_jobs().await {
                error!("Unable to push jobs to nats: {err}");
            }
        });

        Ok(())
    }
}
