use crate::{
    job::{producer::JobProducer, queue::JobQueue},
    DalContext,
};
use async_trait::async_trait;

use super::{JobQueueProcessor, JobQueueProcessorResult};

#[derive(Clone, Debug)]
pub struct TestNullProcessor {
    queue: JobQueue,
}

impl TestNullProcessor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(Self {
            queue: JobQueue::new(),
        })
    }
}

#[async_trait]
impl JobQueueProcessor for TestNullProcessor {
    async fn enqueue_job(
        &self,
        job: Box<dyn JobProducer + Send + Sync>,
        _ctx: &DalContext<'_, '_>,
    ) {
        self.queue.enqueue_job(job).await;
    }

    async fn process_queue(&self) -> JobQueueProcessorResult<()> {
        if self.queue.is_empty().await {
            Ok(())
        } else {
            panic!("ended transaction with non-empty job queue");
        }
    }
}
