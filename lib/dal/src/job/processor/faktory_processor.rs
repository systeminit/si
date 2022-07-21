use async_trait::async_trait;
use faktory_async::Client;
use std::{collections::VecDeque, convert::TryInto, sync::Arc};
use telemetry::prelude::*;
use tokio::sync::Mutex;

use crate::job::producer::JobProducer;

use super::{JobQueueProcessor, JobQueueProcessorResult};

#[derive(Clone, Debug)]
pub struct FaktoryProcessor {
    client: Client,
}

impl FaktoryProcessor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl JobQueueProcessor for FaktoryProcessor {
    async fn process_queue(
        &self,
        queue: Arc<Mutex<VecDeque<Box<dyn JobProducer + Send + Sync>>>>,
    ) -> JobQueueProcessorResult<()> {
        while let Some(job) = queue.lock().await.pop_front() {
            let faktory_job = job.try_into()?;
            if let Err(err) = self.client.push(faktory_job).await {
                error!("Faktory push failed, some jobs will be dropped");
                return Err(err)?;
            }
        }

        Ok(())
    }
}
