use super::producer::JobProducer;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct JobQueueElement {
    pub job: Box<dyn JobProducer + Send + Sync>,
    pub wait_for_execution: bool,
}

#[derive(Debug, Clone, Default)]
pub struct JobQueue {
    queue: Arc<Mutex<VecDeque<JobQueueElement>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
        }
    }

    pub async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        let mut lock = self.queue.lock().await;
        let already_enqueued = lock.iter().any(|j| j.job.identity() == job.identity());

        if !already_enqueued {
            lock.push_back(JobQueueElement {
                job,
                wait_for_execution: false,
            });
        }
    }

    pub async fn enqueue_blocking_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        let mut lock = self.queue.lock().await;
        let already_enqueued = lock.iter_mut().find(|j| j.job.identity() == job.identity());

        if let Some(enqueued) = already_enqueued {
            enqueued.wait_for_execution = true;
        } else {
            lock.push_back(JobQueueElement {
                job,
                wait_for_execution: true,
            });
        }
    }

    pub async fn fetch_job(&self) -> Option<JobQueueElement> {
        self.queue.lock().await.pop_front()
    }

    pub async fn empty(&self) -> VecDeque<JobQueueElement> {
        std::mem::take(&mut *self.queue.lock().await)
    }

    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }
}
