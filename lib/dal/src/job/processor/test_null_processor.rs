use std::{collections::VecDeque, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::job::producer::JobProducer;

use super::{JobQueueProcessor, JobQueueProcessorResult};

#[derive(Clone, Debug)]
pub struct TestNullProcessor {}

impl TestNullProcessor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(Self {})
    }
}

#[async_trait]
impl JobQueueProcessor for TestNullProcessor {
    async fn process_queue(
        &self,
        queue: Arc<Mutex<VecDeque<Box<dyn JobProducer + Send + Sync>>>>,
    ) -> JobQueueProcessorResult<()> {
        if queue.lock().await.is_empty() {
            Ok(())
        } else {
            panic!("ended transaction with non-empty job queue");
        }
    }
}
