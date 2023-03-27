use async_trait::async_trait;

use super::{JobQueueProcessor, JobQueueProcessorResult};
use crate::{
    job::{
        consumer::{JobConsumer, JobInfo},
        definition::{DependentValuesUpdate, FixesJob, RefreshJob},
        producer::{BlockingJobResult, JobProducer},
    },
    DalContext,
};

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
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>, ctx: &DalContext) {
        // hack, sync processor doesn't work with transactions being commited mid job, like
        // dependent values update does, so multiple jobs can access each other's data as they run
        // in parallel in pinga, sychronized by council. The tests are the only users of sync
        // processor currently, and they break without this because the transaction from the shared
        // ctx will be commited before the test ends
        let job_info = JobInfo::try_from(job).expect("unable to get JobInfo");
        let mut job = match job_info.kind() {
            stringify!(DependentValuesUpdate) => Box::new(
                DependentValuesUpdate::try_from(job_info)
                    .expect("unable to get DepedentValuesUpdate"),
            )
                as Box<dyn JobConsumer + Send + Sync>,
            stringify!(FixesJob) => {
                Box::new(FixesJob::try_from(job_info).expect("unable to obtain FixesJob"))
                    as Box<dyn JobConsumer + Send + Sync>
            }
            stringify!(RefreshJob) => {
                Box::new(RefreshJob::try_from(job_info).expect("unable to obtain RefreshJob"))
                    as Box<dyn JobConsumer + Send + Sync>
            }
            kind => panic!("job kind not supported: {kind:?}"),
        };

        job.set_sync();
        job.run(ctx)
            .await
            .unwrap_or_else(|e| panic!("Failure processing background job:\n  {job:?}\n\n{e}"));
    }

    async fn enqueue_blocking_job(
        &self,
        job: Box<dyn JobProducer + Send + Sync>,
        ctx: &DalContext,
    ) {
        self.enqueue_job(job, ctx).await
    }

    async fn block_on_job(
        &self,
        job: Box<dyn JobProducer + Send + Sync>,
        ctx: &DalContext,
    ) -> BlockingJobResult {
        self.enqueue_job(job, ctx).await;
        Ok(())
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        Ok(())
    }
}
