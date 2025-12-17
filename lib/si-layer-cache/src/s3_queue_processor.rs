use std::{
    collections::HashMap,
    hash::{
        Hash,
        Hasher,
    },
    sync::Arc,
};

use aws_sdk_s3::Client;
use crossbeam_queue::SegQueue;
use telemetry::prelude::*;
use telemetry_utils::gauge;
use tokio::{
    sync::{
        Notify,
        mpsc,
    },
    task::JoinHandle,
};
use ulid::Ulid;

use crate::{
    s3_disk_store::S3DiskStore,
    s3_worker::Worker,
};

/// Work item for S3 upload queue
#[derive(Debug, Clone)]
pub struct WorkItem {
    pub ulid: Ulid,
    pub prefix: String,
}

impl WorkItem {
    pub fn new(ulid: Ulid, prefix: String) -> Self {
        Self { ulid, prefix }
    }
}

/// Configuration for initializing S3QueueProcessor with queue state
pub struct ProcessorQueueSetup {
    pub rx: mpsc::UnboundedReceiver<WorkItem>,
    pub notify: Arc<Notify>,
    pub initial_work_items: Vec<WorkItem>,
}

/// Information for a single worker
struct WorkerInfo {
    handle: JoinHandle<()>,
    queue: Arc<SegQueue<WorkItem>>,
    notify: Arc<Notify>,
}

pub struct S3QueueProcessor {
    /// Worker information (worker_id -> info)
    workers: HashMap<usize, WorkerInfo>,
    /// Number of workers
    num_workers: usize,
    /// Incoming work channel receiver
    rx: mpsc::UnboundedReceiver<WorkItem>,
    /// Cache name for metrics/logging
    cache_name: String,
    /// Shutdown signal
    shutdown: Arc<Notify>,
}

/// Fatal error when no workers are available for persistence
#[derive(Debug, thiserror::Error)]
#[error("No workers available - cannot persist to S3")]
pub struct NoWorkersError;

impl S3QueueProcessor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        disk_store: Arc<S3DiskStore>,
        num_workers: usize,
        max_parallel_per_worker: usize,
        s3_client: Client,
        bucket_name: String,
        cache_name: String,
        rx: mpsc::UnboundedReceiver<WorkItem>,
        initial_work_items: Vec<WorkItem>,
    ) -> Result<Self, NoWorkersError> {
        let mut workers = HashMap::new();
        let shutdown = Arc::new(Notify::new());

        // Create workers
        for worker_id in 0..num_workers {
            let work_queue = Arc::new(SegQueue::new());
            let work_available = Arc::new(Notify::new());

            let worker = Worker::new(
                worker_id,
                Arc::clone(&work_queue),
                Arc::clone(&work_available),
                max_parallel_per_worker,
                Arc::clone(&disk_store),
                s3_client.clone(),
                bucket_name.clone(),
                cache_name.clone(),
                Arc::clone(&shutdown),
            );

            let handle = tokio::spawn(worker.run());

            workers.insert(
                worker_id,
                WorkerInfo {
                    handle,
                    queue: work_queue,
                    notify: work_available,
                },
            );
        }

        let processor = Self {
            workers,
            num_workers,
            rx,
            cache_name,
            shutdown,
        };

        // Route initial work items to workers
        for work_item in initial_work_items {
            let worker_info = processor.worker_for_prefix(&work_item.prefix)?;
            worker_info.queue.push(work_item);
            worker_info.notify.notify_one();
        }

        Ok(processor)
    }

    pub fn shutdown_handle(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown)
    }

    /// Get worker for a prefix using consistent hashing
    ///
    /// If the expected worker is not found, log an error and fall back to any available worker.
    /// Only return an error if there are no workers at all, since no uploads are possible. This
    /// fallback should only be necessary if a worker unexpectedly exits for some reason.
    ///
    fn worker_for_prefix(&self, prefix: &str) -> Result<&WorkerInfo, NoWorkersError> {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        prefix.hash(&mut hasher);
        let worker_id = (hasher.finish() as usize) % self.num_workers;

        // Try to get the expected worker
        if let Some(worker_info) = self.workers.get(&worker_id) {
            return Ok(worker_info);
        }

        // Expected worker not found - really shouldn't happen as workers should
        // only ever go away on shutdown.
        error!(
            cache_name = %self.cache_name,
            expected_worker_id = worker_id,
            num_workers = self.num_workers,
            actual_workers = self.workers.len(),
            prefix = prefix,
            "Expected worker not found, attempting another worker"
        );

        // Fall back to any available worker
        self.workers.values().next().ok_or(NoWorkersError)
    }

    /// Process incoming work and route to workers
    pub async fn process_queue(mut self) {
        info!(cache_name = %self.cache_name, num_workers = self.num_workers, "S3QueueProcessor starting");

        // Process incoming work with periodic metrics reporting
        loop {
            tokio::select! {
                // Timeout ensures we report metrics at least every minute during idle periods.
                work_result = tokio::time::timeout(
                    std::time::Duration::from_secs(60),
                    self.rx.recv()
                ) => {
                    match work_result {
                        Ok(Some(work_item)) => {
                            // Route work to appropriate worker
                            match self.worker_for_prefix(&work_item.prefix) {
                                Ok(worker_info) => {
                                    worker_info.queue.push(work_item);
                                    worker_info.notify.notify_one();
                                }
                                Err(NoWorkersError) => {
                                    error!(
                                        cache_name = %self.cache_name,
                                        "No S3 upload workers available - unable to persist items"
                                    );
                                    break;
                                }
                            }
                        }
                        Ok(None) => {
                            // Channel closed
                            info!(cache_name = %self.cache_name, "Work channel closed, shutting down");
                            break;
                        }
                        Err(_timeout) => {
                            // Timeout elapsed - continue to metrics reporting
                        }
                    }
                }
                _ = self.shutdown.notified() => {
                    info!(cache_name = %self.cache_name, "S3QueueProcessor shutting down");
                    break;
                }
            }

            let total_depth: usize = self
                .workers
                .values()
                .map(|worker_info| worker_info.queue.len())
                .sum();

            gauge!(
                s3_write_queue_depth = total_depth,
                cache_name = &self.cache_name,
                backend = "s3"
            );
        }

        // Wait for workers to finish
        for (worker_id, worker_info) in self.workers {
            if let Err(e) = worker_info.handle.await {
                error!(
                    worker_id = worker_id,
                    cache_name = %self.cache_name,
                    error = ?e,
                    "Worker task failed"
                );
            }
        }

        info!(cache_name = %self.cache_name, "S3QueueProcessor stopped");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crossbeam_queue::SegQueue;
    use tokio::sync::{
        Notify,
        mpsc,
    };

    use super::*;

    fn create_test_processor(num_workers: usize) -> S3QueueProcessor {
        let mut workers = HashMap::new();
        let shutdown = Arc::new(Notify::new());
        let (_tx, rx) = mpsc::unbounded_channel();

        for worker_id in 0..num_workers {
            workers.insert(
                worker_id,
                WorkerInfo {
                    handle: tokio::spawn(async {}),
                    queue: Arc::new(SegQueue::new()),
                    notify: Arc::new(Notify::new()),
                },
            );
        }

        S3QueueProcessor {
            workers,
            num_workers,
            rx,
            cache_name: "test".to_string(),
            shutdown,
        }
    }

    #[tokio::test]
    async fn test_worker_for_prefix_same_prefix_same_worker() {
        let processor = create_test_processor(10);
        let prefix = "ab/cd/ef/";

        let worker1 = processor.worker_for_prefix(prefix).unwrap();
        let worker2 = processor.worker_for_prefix(prefix).unwrap();

        assert!(
            std::ptr::eq(worker1, worker2),
            "Same prefix should always route to same worker"
        );
    }

    #[tokio::test]
    async fn test_worker_for_prefix_distribution() {
        let processor = create_test_processor(10);
        let mut worker_ids = std::collections::HashSet::new();

        let prefixes = vec![
            "ab/cd/ef/",
            "12/34/56/",
            "fe/dc/ba/",
            "00/11/22/",
            "99/88/77/",
        ];

        for prefix in prefixes {
            let worker = processor.worker_for_prefix(prefix).unwrap();
            // Calculate worker_id from pointer comparison
            for (id, info) in &processor.workers {
                if std::ptr::eq(worker, info) {
                    worker_ids.insert(*id);
                    break;
                }
            }
        }

        assert!(
            worker_ids.len() > 1,
            "Different prefixes should distribute across workers"
        );
    }
}
