use si_layer_cache::retry_queue::{
    RetryQueueConfig,
    RetryQueueManager,
};
use tempfile::TempDir;

#[tokio::test]
async fn test_scan_existing_queues() {
    let temp_dir = TempDir::new().unwrap();
    let config = RetryQueueConfig {
        base_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let mut manager = RetryQueueManager::new(config);

    // Scanning empty directory should succeed
    manager
        .scan_existing_queues(&["test_cache"])
        .await
        .unwrap();
}
