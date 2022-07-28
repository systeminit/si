use async_trait::async_trait;

use super::{JobQueueProcessor, JobQueueProcessorResult};
use crate::{job::producer::JobProducer, DalContext};

#[derive(Clone, Debug, Default)]
pub struct SyncProcessor {}

/// The `SyncProcessor` executes jobs inline, without sending to another
/// queue or messaging service for async processing.
impl SyncProcessor {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl JobQueueProcessor for SyncProcessor {
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>, ctx: &DalContext<'_, '_>) {
        job.run(ctx)
            .await
            .expect("Failure processing background job");
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        Ok(())
    }
}
