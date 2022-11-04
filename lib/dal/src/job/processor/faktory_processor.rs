use async_trait::async_trait;
use si_data_faktory::Client;
use std::convert::TryInto;
use telemetry::prelude::*;
use tokio::sync::mpsc;

use crate::{
    job::{producer::JobProducer, queue::JobQueue},
    DalContext,
};

use super::{JobQueueProcessor, JobQueueProcessorResult};

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
            let faktory_job = job.try_into()?;
            if let Err(err) = self.client.push(faktory_job).await {
                error!("Faktory push failed, some jobs will be dropped");
                return Err(err.into());
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
