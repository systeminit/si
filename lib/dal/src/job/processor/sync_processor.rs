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
    async fn enqueue_job(&self, mut job: Box<dyn JobProducer + Send + Sync>, ctx: &DalContext) {
        // hack, sync processor doesn't work with transactions being commited mid job, like dependent values update does so multiple jobs can access eachother datas as they run in parallel in pinga, sychronized by council
        // the tests are the only users of sync processor currently, and they break without this because the transaction from the shared ctx will be commited before the test ends
        job.set_sync();
        job.run(ctx)
            .await
            .unwrap_or_else(|e| panic!("Failure processing background job:\n  {job:?}\n\n{e}"));
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        Ok(())
    }
}
