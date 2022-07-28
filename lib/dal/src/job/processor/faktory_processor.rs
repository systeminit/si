use async_trait::async_trait;
use faktory_async::Client;
use std::convert::TryInto;
use telemetry::prelude::*;

use crate::job::{producer::JobProducer, queue::JobQueue};

use super::{JobQueueProcessor, JobQueueProcessorResult};

#[derive(Clone, Debug)]
pub struct FaktoryProcessor {
    client: Client,
    queue: JobQueue,
}

impl FaktoryProcessor {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            queue: JobQueue::new(),
        }
    }
}

#[async_trait]
impl JobQueueProcessor for FaktoryProcessor {
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        self.queue.enqueue_job(job).await
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        while let Some(job) = self.queue.fetch_job().await {
            let faktory_job = job.try_into()?;
            if let Err(err) = self.client.push(faktory_job).await {
                error!("Faktory push failed, some jobs will be dropped");
                return Err(err)?;
            }
        }

        Ok(())
    }
}
