use async_trait::async_trait;
use faktory::Producer;
use std::{collections::VecDeque, convert::TryInto, sync::Arc};
use tokio::sync::Mutex;

use crate::job::producer::JobProducer;

use super::{JobQueueProcessor, JobQueueProcessorResult};

#[derive(Clone)]
pub struct FaktoryProcessor {
    url: String,
}

impl std::fmt::Debug for FaktoryProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FaktoryProcessor")
            .field("inner", &"<TcpStream>")
            .finish()
    }
}

impl FaktoryProcessor {
    pub fn new(url: &str) -> JobQueueProcessorResult<Self> {
        Ok(Self {
            url: url.to_string(),
        })
    }
}

#[async_trait]
impl JobQueueProcessor for FaktoryProcessor {
    async fn process_queue(
        &self,
        queue: Arc<Mutex<VecDeque<Box<dyn JobProducer + Send + Sync>>>>,
    ) -> JobQueueProcessorResult<()> {
        let mut connection = Producer::connect(Some(&self.url))?;

        while let Some(job) = queue.lock().await.pop_front() {
            let faktory_job = job.try_into()?;
            connection.enqueue(faktory_job)?;
        }

        Ok(())
    }
}
