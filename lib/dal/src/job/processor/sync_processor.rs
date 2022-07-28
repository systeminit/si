use async_trait::async_trait;
use std::sync::Arc;
use crate::DalContextBuilder;

use crate::job::{queue::JobQueue, producer::JobProducer};
use super::{JobQueueProcessor, JobQueueProcessorResult};

#[derive(Clone, Debug)]
pub struct SyncProcessor {
    queue: JobQueue,
    ctx_builder: Option<Arc<DalContextBuilder>>,
}

/// The `SyncProcessor` executes jobs inline, without sending to another
/// queue or messaging service for async processing.
impl SyncProcessor {
    pub fn new() -> Self {
        Self { ctx_builder: None, queue: JobQueue::new() }
    }

    pub fn set_dal_ctx_builder(&mut self, ctx_builder: Arc<DalContextBuilder>) {
        self.ctx_builder = Some(ctx_builder);
    }
}

#[async_trait]
impl JobQueueProcessor for SyncProcessor {
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        self.queue.enqueue_job(job).await
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        let ctx_builder = self.ctx_builder.clone().unwrap();
        while let Some(job) = self.queue.fetch_job().await {
            job.run_job(ctx_builder.clone()).await.expect("Failure processing background job");
        }

        Ok(())
    }
}

