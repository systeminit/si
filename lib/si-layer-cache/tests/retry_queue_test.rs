use std::sync::Arc;

use si_events::{
    Actor,
    ChangeSetId,
    Tenancy,
    WorkspacePk,
};
use si_layer_cache::{
    event::{
        LayeredEvent,
        LayeredEventKind,
    },
    retry_queue::{
        RetryQueueConfig,
        RetryQueueManager,
        RetryQueueMessage,
    },
};
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

fn create_test_event(cache_name: &str) -> LayeredEvent {
    LayeredEvent::new(
        LayeredEventKind::Raw,
        Arc::new(cache_name.to_string()),
        Arc::from("test-key"),
        Arc::new(vec![1u8, 2, 3]),
        Arc::new("test-sort".to_string()),
        None,
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::System,
    )
}

#[tokio::test]
async fn test_enqueue_and_retrieve() {
    let temp_dir = TempDir::new().unwrap();
    let config = RetryQueueConfig {
        base_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = RetryQueueManager::new(config);
    let shutdown_token = CancellationToken::new();

    let (queue_tx, queue_rx) = mpsc::unbounded_channel();
    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();

    // Spawn manager task
    let manager_handle = tokio::spawn(manager.run(queue_rx, ready_tx, shutdown_token.clone()));

    // Enqueue an event
    let event = create_test_event("test_cache");
    queue_tx
        .send(RetryQueueMessage::Enqueue(event.clone()))
        .unwrap();

    // Should receive it as ready (backoff starts at 100ms, but immediately ready after enqueue)
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    let result = tokio::time::timeout(tokio::time::Duration::from_secs(2), ready_rx.recv()).await;

    assert!(result.is_ok());
    let (retrieved_event, _handle) = result.unwrap().unwrap();
    assert_eq!(retrieved_event.payload.key, event.payload.key);

    // Cleanup
    shutdown_token.cancel();
    let _ = manager_handle.await;
}

#[tokio::test]
async fn test_mark_success_removes_from_queue() {
    let temp_dir = TempDir::new().unwrap();
    let config = RetryQueueConfig {
        base_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = RetryQueueManager::new(config);
    let shutdown_token = CancellationToken::new();

    let (queue_tx, queue_rx) = mpsc::unbounded_channel();
    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();

    // Spawn manager task
    let manager_handle = tokio::spawn(manager.run(queue_rx, ready_tx, shutdown_token.clone()));

    // Enqueue event
    let event = create_test_event("test_cache");
    queue_tx.send(RetryQueueMessage::Enqueue(event)).unwrap();

    // Receive ready retry
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    let (_, handle) = tokio::time::timeout(tokio::time::Duration::from_secs(2), ready_rx.recv())
        .await
        .unwrap()
        .unwrap();

    // Mark as successful
    queue_tx
        .send(RetryQueueMessage::MarkSuccess(handle))
        .unwrap();

    // Give manager time to process the MarkSuccess message and ensure the item is removed
    // We need to wait longer than the initial backoff (100ms) to ensure no retry happens
    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;

    // Should not receive another retry - short timeout since we already waited
    let result =
        tokio::time::timeout(tokio::time::Duration::from_millis(50), ready_rx.recv()).await;

    assert!(result.is_err(), "Should timeout - no retry available");

    // Cleanup
    shutdown_token.cancel();
    let _ = manager_handle.await;
}

#[tokio::test]
async fn test_backoff_increases_on_failure() {
    let temp_dir = TempDir::new().unwrap();
    let config = RetryQueueConfig {
        base_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = RetryQueueManager::new(config);
    let shutdown_token = CancellationToken::new();

    let (queue_tx, queue_rx) = mpsc::unbounded_channel();
    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();

    // Spawn manager task
    let manager_handle = tokio::spawn(manager.run(queue_rx, ready_tx, shutdown_token.clone()));

    // Enqueue event
    let event = create_test_event("test_cache");
    queue_tx.send(RetryQueueMessage::Enqueue(event)).unwrap();

    // Receive first retry
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    let (_, handle) = tokio::time::timeout(tokio::time::Duration::from_secs(2), ready_rx.recv())
        .await
        .unwrap()
        .unwrap();

    // Mark as failed with retryable error - use PgPoolError since we can easily construct it
    let io_error = std::io::Error::other("test error");
    let pg_pool_error = si_data_pg::PgPoolError::CreateCertificate(io_error);
    let error = si_layer_cache::LayerDbError::PgPool(pg_pool_error);
    queue_tx
        .send(RetryQueueMessage::MarkRetryableFailure(handle, error))
        .unwrap();

    // Give manager time to process the failure message
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Should not be ready immediately (backoff applied - initial backoff is 100ms)
    let result =
        tokio::time::timeout(tokio::time::Duration::from_millis(80), ready_rx.recv()).await;
    assert!(result.is_err(), "Should not be ready yet due to backoff");

    // After waiting for backoff (100ms initial), should be available
    let result =
        tokio::time::timeout(tokio::time::Duration::from_millis(200), ready_rx.recv()).await;
    assert!(result.is_ok(), "Should be ready after backoff period");

    // Cleanup
    shutdown_token.cancel();
    let _ = manager_handle.await;
}

#[tokio::test]
async fn test_scan_existing_queues() {
    let temp_dir = TempDir::new().unwrap();
    let config = RetryQueueConfig {
        base_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    // Create first manager and enqueue
    let mut manager1 = RetryQueueManager::new(config.clone());
    let event = create_test_event("test_cache");
    manager1.enqueue(event).await.unwrap();

    // Create second manager, scan, and spawn
    let mut manager2 = RetryQueueManager::new(config);
    manager2
        .scan_existing_queues(&["test_cache"])
        .await
        .unwrap();

    let shutdown_token = CancellationToken::new();
    let (_queue_tx, queue_rx) = mpsc::unbounded_channel();
    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();

    // Spawn manager task
    let manager_handle = tokio::spawn(manager2.run(queue_rx, ready_tx, shutdown_token.clone()));

    // Should find the queued item immediately (retries on startup)
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(2), ready_rx.recv()).await;

    assert!(result.is_ok(), "Should find existing queue item");

    // Cleanup
    shutdown_token.cancel();
    let _ = manager_handle.await;
}
