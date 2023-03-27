use super::producer::JobProducer;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct JobQueue {
    queue: Arc<Mutex<VecDeque<Box<dyn JobProducer + Send + Sync>>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
        }
    }

    pub async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        let mut lock = self.queue.lock().await;

        lock.push_back(job);
    }

    pub async fn fetch_job(&self) -> Option<Box<dyn JobProducer + Send + Sync>> {
        self.queue.lock().await.pop_front()
    }

    pub async fn empty(&self) -> VecDeque<Box<dyn JobProducer + Send + Sync>> {
        std::mem::take(&mut *self.queue.lock().await)
    }

    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }

    pub async fn drain(&self) -> Vec<Box<dyn JobProducer + Send + Sync>> {
        self.queue.lock().await.drain(0..).collect()
    }
}
