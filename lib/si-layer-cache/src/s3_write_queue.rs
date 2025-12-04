use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
};

use telemetry::tracing;
use tokio::sync::{
    Notify,
    mpsc,
};
use ulid::Ulid;

use crate::event::LayeredEvent;

/// Errors that can occur during S3WriteQueue operations
#[derive(Debug, thiserror::Error)]
pub enum S3WriteQueueError {
    #[error("Failed to serialize event")]
    SerializationFailed(#[source] postcard::Error),

    #[error("Failed to deserialize event from file {file_path}")]
    DeserializationFailed {
        file_path: PathBuf,
        #[source]
        source: postcard::Error,
    },

    #[error("Invalid filename: expected valid UTF-8 string")]
    InvalidFilename { path: PathBuf },

    #[error("Invalid ULID in filename: {filename}")]
    InvalidUlid {
        filename: String,
        #[source]
        source: ulid::DecodeError,
    },

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("S3 configuration error: {message}")]
    Configuration { message: String },

    #[error("Scan encountered {error_count} corrupted files, moved to dead letter queue")]
    ScanWithErrors { error_count: usize },

    #[error("Failed to send ULID to processor - processor may have crashed")]
    ProcessorChannelClosed,

    #[error("Failed to read event from disk for ULID {ulid}: {source}")]
    DiskReadFailed {
        ulid: Ulid,
        #[source]
        source: Box<S3WriteQueueError>,
    },
}

#[derive(Debug)]
pub struct S3WriteQueue {
    queue_dir: PathBuf,
    dlq_dir: PathBuf,
    tx: mpsc::UnboundedSender<Ulid>,
    notify: Arc<Notify>,
}

impl S3WriteQueue {
    /// Creates a new S3WriteQueue with persistent disk storage.
    /// Directory structure: {base_path}/{cache_name}_s3_queue/
    /// Dead letter queue: {base_path}/{cache_name}_s3_queue/dead_letter/
    pub fn new(
        base_path: impl AsRef<Path>,
        cache_name: impl AsRef<str>,
        notify: Arc<Notify>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Ulid>), S3WriteQueueError> {
        let queue_dir = base_path
            .as_ref()
            .join(format!("{}_s3_queue", cache_name.as_ref()));
        let dlq_dir = queue_dir.join("dead_letter");

        // Create directories if they don't exist
        fs::create_dir_all(&queue_dir)?;
        fs::create_dir_all(&dlq_dir)?;

        // Create channel for communicating pending ULIDs to processor
        let (tx, rx) = mpsc::unbounded_channel();

        Ok((
            Self {
                queue_dir,
                dlq_dir,
                tx,
                notify,
            },
            rx,
        ))
    }

    /// Enqueues a LayeredEvent to disk with ULID-based filename.
    /// Returns the ULID for tracking.
    /// File is atomically written to ensure durability.
    /// Notifies processor of new item via channel after disk write completes.
    pub fn enqueue(&self, event: &LayeredEvent) -> Result<Ulid, S3WriteQueueError> {
        let ulid = Ulid::new();
        let file_path = self.queue_dir.join(format!("{ulid}.pending"));

        // Serialize event
        let bytes = postcard::to_stdvec(event).map_err(S3WriteQueueError::SerializationFailed)?;

        // Write atomically (write to temp, then rename)
        let temp_path = self.queue_dir.join(format!("{ulid}.tmp"));
        fs::write(&temp_path, &bytes)?;
        fs::rename(&temp_path, &file_path)?;

        // Send ULID to processor after disk write completes
        // If channel is closed, processor crashed - this is a fatal error
        self.tx
            .send(ulid)
            .map_err(|_| S3WriteQueueError::ProcessorChannelClosed)?;

        // Notify processor that item is available
        // Cheap if processor not waiting - just clears notification
        self.notify.notify_one();

        Ok(ulid)
    }

    /// Scans queue directory for pending writes, returns only ULIDs in order.
    /// Used on startup to build initial in-memory index.
    /// Files with invalid ULID filenames are moved to dead letter queue.
    /// Note: File content corruption is NOT detected here - only during read_event().
    /// Returns an error only if no files could be processed.
    pub fn scan_ulids(&self) -> Result<Vec<Ulid>, S3WriteQueueError> {
        let mut ulids = Vec::new();
        let mut errors = Vec::new();

        for entry in fs::read_dir(&self.queue_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .pending files
            if path.extension().and_then(|s| s.to_str()) != Some("pending") {
                continue;
            }

            // Extract ULID from filename
            match self.extract_ulid_from_path(&path) {
                Ok(ulid) => {
                    ulids.push(ulid);
                }
                Err(err) => {
                    // Log the error and attempt to move to dead letter queue
                    tracing::error!(
                        filename = %path.display(),
                        error = %err,
                        "Failed to extract ULID from filename"
                    );
                    errors.push(err);

                    // Try to move file with invalid filename to dead letter queue
                    // Use new ULID since we couldn't parse one from filename
                    let ulid = Ulid::new();

                    if let Err(dlq_err) = self.move_corrupted_file_to_dlq(&path, ulid) {
                        tracing::error!(
                            filename = %path.display(),
                            error = %dlq_err,
                            "Failed to move invalid file to dead letter queue"
                        );
                    }
                }
            }
        }

        // Sort by ULID (chronological order)
        ulids.sort();

        // Report aggregate errors if any occurred
        if !errors.is_empty() {
            tracing::warn!(
                invalid_count = errors.len(),
                "Scan completed with invalid filenames moved to dead letter queue"
            );
            // Return error if ALL files had invalid filenames (nothing successfully loaded)
            if ulids.is_empty() {
                return Err(S3WriteQueueError::ScanWithErrors {
                    error_count: errors.len(),
                });
            }
        }

        Ok(ulids)
    }

    /// Reads a single event from disk by ULID.
    /// Used by processor to load event data for writing to S3.
    /// Returns DiskReadFailed error with ULID context for better error handling.
    pub fn read_event(&self, ulid: Ulid) -> Result<LayeredEvent, S3WriteQueueError> {
        let path = self.queue_dir.join(format!("{ulid}.pending"));

        // Read and deserialize event
        let bytes = fs::read(&path).map_err(|io_err| {
            // Wrap IO error with ULID context
            S3WriteQueueError::DiskReadFailed {
                ulid,
                source: Box::new(S3WriteQueueError::Io(io_err)),
            }
        })?;

        let event: LayeredEvent = postcard::from_bytes(&bytes).map_err(|source| {
            // Wrap deserialization error with ULID context
            S3WriteQueueError::DiskReadFailed {
                ulid,
                source: Box::new(S3WriteQueueError::DeserializationFailed {
                    file_path: path.to_path_buf(),
                    source,
                }),
            }
        })?;

        Ok(event)
    }

    /// Extracts ULID from a pending file path
    fn extract_ulid_from_path(&self, path: &Path) -> Result<Ulid, S3WriteQueueError> {
        let filename = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
            S3WriteQueueError::InvalidFilename {
                path: path.to_path_buf(),
            }
        })?;

        Ulid::from_string(filename).map_err(|source| S3WriteQueueError::InvalidUlid {
            filename: filename.to_string(),
            source,
        })
    }

    /// Moves a corrupted file to the dead letter queue (used during scan)
    fn move_corrupted_file_to_dlq(&self, path: &Path, ulid: Ulid) -> Result<(), S3WriteQueueError> {
        let dest_path = self.dlq_dir.join(format!("{ulid}.corrupted"));
        fs::rename(path, &dest_path)?;
        Ok(())
    }

    /// Removes a completed write from the queue.
    /// Succeeds even if file doesn't exist (idempotent).
    pub fn remove(&self, ulid: Ulid) -> Result<(), S3WriteQueueError> {
        let file_path = self.queue_dir.join(format!("{ulid}.pending"));

        match fs::remove_file(&file_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Idempotent
            Err(e) => Err(e.into()),
        }
    }

    /// Moves a corrupted write to the dead letter queue.
    /// Preserves the event data for debugging.
    pub fn move_to_dlq(
        &self,
        ulid: Ulid,
        error: &S3WriteQueueError,
    ) -> Result<(), S3WriteQueueError> {
        let source_path = self.queue_dir.join(format!("{ulid}.pending"));
        let dest_path = self.dlq_dir.join(format!("{ulid}.corrupted"));

        // Log error details at ERROR level as specified in architecture
        tracing::error!(
            ulid = %ulid,
            error = %error,
            "Moving corrupted queue file to dead letter queue"
        );

        // Move file to DLQ
        fs::rename(&source_path, &dest_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use si_events::{
        Actor,
        ChangeSetId,
        Tenancy,
        WorkspacePk,
    };
    use tempfile::TempDir;
    use tokio::sync::Notify;

    use super::*;
    use crate::event::{
        LayeredEventKind,
        LayeredEventMetadata,
    };

    fn create_test_event() -> LayeredEvent {
        LayeredEvent {
            event_id: crate::event::LayeredEventId::new(),
            event_kind: LayeredEventKind::Raw,
            key: Arc::from("test_key"),
            metadata: LayeredEventMetadata {
                tenancy: Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
                actor: Actor::System,
                timestamp: Utc::now(),
            },
            payload: crate::event::LayeredEventPayload {
                db_name: Arc::new("test_db".to_string()),
                key: Arc::from("test_key"),
                sort_key: Arc::new("test_sort".to_string()),
                value: Arc::new(vec![1, 2, 3, 4]),
            },
            web_events: None,
        }
    }

    fn create_test_queue(
        base_path: impl AsRef<Path>,
        cache_name: impl AsRef<str>,
    ) -> (S3WriteQueue, mpsc::UnboundedReceiver<Ulid>) {
        let notify = Arc::new(Notify::new());
        S3WriteQueue::new(base_path, cache_name, notify).unwrap()
    }

    #[test]
    fn test_new_creates_queue_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let (_queue, _rx) = create_test_queue(base_path, "test_cache");

        // Queue directory should exist
        let expected_dir = base_path.join("test_cache_s3_queue");
        assert!(expected_dir.exists());
        assert!(expected_dir.is_dir());
    }

    #[test]
    fn test_new_creates_dlq_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let (_queue, _rx) = create_test_queue(base_path, "test_cache");

        // DLQ directory should exist
        let expected_dlq = base_path.join("test_cache_s3_queue").join("dead_letter");
        assert!(expected_dlq.exists());
        assert!(expected_dlq.is_dir());
    }

    #[test]
    fn test_enqueue_creates_pending_file() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        // Check file exists with correct name
        let file_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{ulid}.pending"));

        assert!(file_path.exists());
        assert!(file_path.is_file());
    }

    #[test]
    fn test_enqueue_serializes_event() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        // Read file and deserialize
        let file_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{ulid}.pending"));

        let bytes = std::fs::read(&file_path).unwrap();
        let deserialized: LayeredEvent = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.payload.db_name, event.payload.db_name);
    }

    #[test]
    fn test_scan_empty_queue() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let ulids = queue.scan_ulids().unwrap();

        assert_eq!(ulids.len(), 0);
    }

    #[test]
    fn test_scan_returns_all_pending_files() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event1 = create_test_event();
        let event2 = create_test_event();
        let event3 = create_test_event();

        queue.enqueue(&event1).unwrap();
        queue.enqueue(&event2).unwrap();
        queue.enqueue(&event3).unwrap();

        let ulids = queue.scan_ulids().unwrap();

        assert_eq!(ulids.len(), 3);
    }

    #[test]
    fn test_scan_returns_ulid_sorted_order() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();

        // Enqueue with slight delays to ensure different ULIDs
        let ulid1 = queue.enqueue(&event).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = queue.enqueue(&event).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid3 = queue.enqueue(&event).unwrap();

        let ulids = queue.scan_ulids().unwrap();

        // Should be in ULID order (chronological)
        assert_eq!(ulids[0], ulid1);
        assert_eq!(ulids[1], ulid2);
        assert_eq!(ulids[2], ulid3);
    }

    #[test]
    fn test_scan_ignores_non_pending_files() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        queue.enqueue(&event).unwrap();

        // Create a non-pending file in the queue directory
        let queue_dir = temp_dir.path().join("test_s3_queue");
        std::fs::write(queue_dir.join("readme.txt"), "test").unwrap();
        std::fs::write(queue_dir.join("other.tmp"), "test").unwrap();

        let ulids = queue.scan_ulids().unwrap();

        // Should only find the one .pending file
        assert_eq!(ulids.len(), 1);
    }

    #[test]
    fn test_remove_deletes_file() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        // File should exist
        let file_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{ulid}.pending"));
        assert!(file_path.exists());

        // Remove it
        queue.remove(ulid).unwrap();

        // File should be gone
        assert!(!file_path.exists());
    }

    #[test]
    fn test_remove_nonexistent_file_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let ulid = Ulid::new();

        // Removing non-existent file should not error
        let result = queue.remove(ulid);
        assert!(result.is_ok());
    }

    #[test]
    fn test_move_to_dlq_moves_file() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        let original_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{ulid}.pending"));
        assert!(original_path.exists());

        // Move to DLQ
        let error = S3WriteQueueError::DeserializationFailed {
            file_path: original_path.clone(),
            source: postcard::Error::DeserializeUnexpectedEnd,
        };
        queue.move_to_dlq(ulid, &error).unwrap();

        // Original should be gone
        assert!(!original_path.exists());

        // Should be in DLQ with .corrupted extension
        let dlq_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join("dead_letter")
            .join(format!("{ulid}.corrupted"));
        assert!(dlq_path.exists());
    }

    #[test]
    fn test_move_to_dlq_preserves_event_data() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        let original_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{ulid}.pending"));
        let error = S3WriteQueueError::DeserializationFailed {
            file_path: original_path,
            source: postcard::Error::DeserializeUnexpectedEnd,
        };
        queue.move_to_dlq(ulid, &error).unwrap();

        // Read from DLQ and verify data preserved
        let dlq_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join("dead_letter")
            .join(format!("{ulid}.corrupted"));

        let bytes = std::fs::read(&dlq_path).unwrap();
        let deserialized: LayeredEvent = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.payload.db_name, event.payload.db_name);
    }

    #[test]
    fn test_scan_ulids_includes_files_with_corrupted_content() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        // Enqueue valid events
        let event1 = create_test_event();
        let event2 = create_test_event();
        let ulid1 = queue.enqueue(&event1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = queue.enqueue(&event2).unwrap();

        // Create a file with valid ULID filename but corrupted content
        let corrupted_ulid = Ulid::new();
        let corrupted_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{corrupted_ulid}.pending"));
        std::fs::write(&corrupted_path, b"corrupted data").unwrap();

        // scan_ulids() only reads filenames, so it includes all files with valid ULID names
        let ulids = queue.scan_ulids().unwrap();

        // Should have all 3 ULIDs (including the one with corrupted content)
        assert_eq!(ulids.len(), 3);
        assert!(ulids.contains(&ulid1));
        assert!(ulids.contains(&ulid2));
        assert!(ulids.contains(&corrupted_ulid));

        // Corrupted file should still exist (not moved to DLQ yet)
        assert!(corrupted_path.exists());

        // Corruption will be detected when read_event() is called
        let result = queue.read_event(corrupted_ulid);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_with_invalid_filename() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        // Enqueue a valid event
        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        // Create a file with invalid ULID in filename
        let invalid_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join("not-a-ulid.pending");
        std::fs::write(&invalid_path, b"some data").unwrap();

        // Scan should succeed and return the valid ULID
        let ulids = queue.scan_ulids().unwrap();

        // Should have loaded the 1 valid ULID
        assert_eq!(ulids.len(), 1);
        assert_eq!(ulids[0], ulid);

        // Invalid file should be moved to DLQ
        assert!(!invalid_path.exists());
        // Check that something was moved to DLQ
        let dlq_dir = temp_dir.path().join("test_s3_queue").join("dead_letter");
        let dlq_entries: Vec<_> = std::fs::read_dir(&dlq_dir)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(!dlq_entries.is_empty(), "Expected corrupted file in DLQ");
    }

    #[test]
    fn test_scan_ulids_all_files_invalid_filenames() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        // Create multiple files with invalid ULID filenames
        for i in 0..3 {
            let invalid_path = temp_dir
                .path()
                .join("test_s3_queue")
                .join(format!("not-a-ulid-{i}.pending"));
            std::fs::write(&invalid_path, format!("some data {i}")).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // scan_ulids() should fail with ScanWithErrors since all filenames are invalid
        let result = queue.scan_ulids();
        assert!(result.is_err());
        match result {
            Err(S3WriteQueueError::ScanWithErrors { error_count }) => {
                assert_eq!(error_count, 3);
            }
            _ => panic!("Expected ScanWithErrors"),
        }

        // All invalid filename files should be in DLQ
        let dlq_dir = temp_dir.path().join("test_s3_queue").join("dead_letter");
        let dlq_count = std::fs::read_dir(&dlq_dir).unwrap().count();
        assert_eq!(dlq_count, 3);
    }

    #[test]
    fn test_scan_ulids_mixed_valid_and_corrupted_content() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        // Create a mix of valid files and file with corrupted content
        let event1 = create_test_event();
        let ulid1 = queue.enqueue(&event1).unwrap();

        let corrupted_ulid = Ulid::new();
        let corrupted_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{corrupted_ulid}.pending"));
        std::fs::write(&corrupted_path, b"corrupted").unwrap();

        let event2 = create_test_event();
        let ulid2 = queue.enqueue(&event2).unwrap();

        // scan_ulids() only reads filenames, so includes all valid ULID filenames
        let ulids = queue.scan_ulids().unwrap();

        // Should have all 3 ULIDs (content corruption not detected by scan_ulids)
        assert_eq!(ulids.len(), 3);
        assert!(ulids.contains(&ulid1));
        assert!(ulids.contains(&ulid2));
        assert!(ulids.contains(&corrupted_ulid));

        // Corrupted file should still exist (not moved to DLQ yet)
        assert!(corrupted_path.exists());

        // Verify we can read the valid events
        assert!(queue.read_event(ulid1).is_ok());
        assert!(queue.read_event(ulid2).is_ok());
        // Verify corruption detected on read
        assert!(queue.read_event(corrupted_ulid).is_err());
    }

    #[test]
    fn test_scan_returns_count() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        assert_eq!(queue.scan_ulids().unwrap().len(), 0);

        queue.enqueue(&create_test_event()).unwrap();
        assert_eq!(queue.scan_ulids().unwrap().len(), 1);

        queue.enqueue(&create_test_event()).unwrap();
        assert_eq!(queue.scan_ulids().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_enqueue_sends_ulid_to_channel() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, mut rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        // Should be able to receive the ULID from channel
        let received_ulid = rx.try_recv().unwrap();
        assert_eq!(received_ulid, ulid);
    }

    #[tokio::test]
    async fn test_enqueue_multiple_items_to_channel() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, mut rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();

        // Enqueue 3 items
        let ulid1 = queue.enqueue(&event).unwrap();
        let ulid2 = queue.enqueue(&event).unwrap();
        let ulid3 = queue.enqueue(&event).unwrap();

        // All 3 should be in channel in order
        assert_eq!(rx.try_recv().unwrap(), ulid1);
        assert_eq!(rx.try_recv().unwrap(), ulid2);
        assert_eq!(rx.try_recv().unwrap(), ulid3);

        // Channel should be empty now
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_enqueue_fails_when_processor_channel_closed() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, rx) = create_test_queue(temp_dir.path(), "test");

        // Drop receiver to close channel (simulates processor crash)
        drop(rx);

        let event = create_test_event();
        let result = queue.enqueue(&event);

        // Should fail with ProcessorChannelClosed error
        assert!(result.is_err());
        match result {
            Err(S3WriteQueueError::ProcessorChannelClosed) => {
                // Expected error type
            }
            _ => panic!("Expected ProcessorChannelClosed error"),
        }
    }

    #[test]
    fn test_scan_ulids_returns_ulids_only() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();

        // Enqueue with slight delays to ensure different ULIDs
        let ulid1 = queue.enqueue(&event).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = queue.enqueue(&event).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid3 = queue.enqueue(&event).unwrap();

        let ulids = queue.scan_ulids().unwrap();

        // Should return just ULIDs in chronological order
        assert_eq!(ulids.len(), 3);
        assert_eq!(ulids[0], ulid1);
        assert_eq!(ulids[1], ulid2);
        assert_eq!(ulids[2], ulid3);
    }

    #[test]
    fn test_scan_ulids_empty_queue() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let ulids = queue.scan_ulids().unwrap();

        assert_eq!(ulids.len(), 0);
    }

    #[test]
    fn test_read_event_by_ulid() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        // Read event back from disk by ULID
        let read_event = queue.read_event(ulid).unwrap();

        assert_eq!(read_event.payload.db_name, event.payload.db_name);
        assert_eq!(read_event.key, event.key);
    }

    #[test]
    fn test_read_event_nonexistent_ulid() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        let nonexistent_ulid = Ulid::new();

        // Should fail with DiskReadFailed wrapping IO error (file not found)
        let result = queue.read_event(nonexistent_ulid);
        assert!(result.is_err());
        match result {
            Err(S3WriteQueueError::DiskReadFailed { ulid, source }) => {
                assert_eq!(ulid, nonexistent_ulid);
                // Check that the inner error is an IO error
                match *source {
                    S3WriteQueueError::Io(ref io_err) => {
                        assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
                    }
                    _ => panic!("Expected inner Io error"),
                }
            }
            _ => panic!("Expected DiskReadFailed error"),
        }
    }

    #[test]
    fn test_scan_ulids_handles_invalid_filenames() {
        let temp_dir = TempDir::new().unwrap();
        let (queue, _rx) = create_test_queue(temp_dir.path(), "test");

        // Enqueue valid events
        let event1 = create_test_event();
        let event2 = create_test_event();
        let ulid1 = queue.enqueue(&event1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = queue.enqueue(&event2).unwrap();

        // Create a file with invalid ULID filename
        let invalid_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join("not-a-valid-ulid.pending");
        std::fs::write(&invalid_path, b"some data").unwrap();

        // Scan should succeed and return the valid ULIDs (skipping invalid filename)
        let ulids = queue.scan_ulids().unwrap();

        // Should have loaded the 2 valid ULIDs
        assert_eq!(ulids.len(), 2);
        assert_eq!(ulids[0], ulid1);
        assert_eq!(ulids[1], ulid2);

        // Invalid filename file should be moved to DLQ
        assert!(!invalid_path.exists());
        // Note: Can't easily check DLQ path since filename was invalid and we use new ULID
    }
}
