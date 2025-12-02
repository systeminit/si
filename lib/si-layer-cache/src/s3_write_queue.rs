use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use telemetry::tracing;
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
}

#[derive(Debug)]
pub struct S3WriteQueue {
    queue_dir: PathBuf,
    dlq_dir: PathBuf,
}

impl S3WriteQueue {
    /// Creates a new S3WriteQueue with persistent disk storage.
    /// Directory structure: {base_path}/{cache_name}_s3_queue/
    /// Dead letter queue: {base_path}/{cache_name}_s3_queue/dead_letter/
    pub fn new(
        base_path: impl AsRef<Path>,
        cache_name: impl AsRef<str>,
    ) -> Result<Self, S3WriteQueueError> {
        let queue_dir = base_path
            .as_ref()
            .join(format!("{}_s3_queue", cache_name.as_ref()));
        let dlq_dir = queue_dir.join("dead_letter");

        // Create directories if they don't exist
        fs::create_dir_all(&queue_dir)?;
        fs::create_dir_all(&dlq_dir)?;

        Ok(Self { queue_dir, dlq_dir })
    }

    /// Enqueues a LayeredEvent to disk with ULID-based filename.
    /// Returns the ULID for tracking.
    /// File is atomically written to ensure durability.
    pub fn enqueue(&self, event: &LayeredEvent) -> Result<Ulid, S3WriteQueueError> {
        let ulid = Ulid::new();
        let file_path = self.queue_dir.join(format!("{ulid}.pending"));

        // Serialize event
        let bytes = postcard::to_stdvec(event).map_err(S3WriteQueueError::SerializationFailed)?;

        // Write atomically (write to temp, then rename)
        let temp_path = self.queue_dir.join(format!("{ulid}.tmp"));
        fs::write(&temp_path, &bytes)?;
        fs::rename(&temp_path, &file_path)?;

        Ok(ulid)
    }

    /// Scans queue directory for pending writes, returns them in ULID order.
    /// Used on startup to load existing queue.
    /// Corrupted files are automatically moved to the dead letter queue.
    /// Returns an error only if no files could be processed due to corrupted data.
    pub fn scan(&self) -> Result<Vec<(Ulid, LayeredEvent)>, S3WriteQueueError> {
        let mut entries = Vec::new();
        let mut errors = Vec::new();

        for entry in fs::read_dir(&self.queue_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .pending files
            if path.extension().and_then(|s| s.to_str()) != Some("pending") {
                continue;
            }

            // Process this file, collecting errors instead of failing
            match self.scan_single_file(&path) {
                Ok((ulid, event)) => {
                    entries.push((ulid, event));
                }
                Err(err) => {
                    // Log the error and attempt to move to DLQ
                    tracing::error!("Failed to process queue file {}: {}", path.display(), err);
                    errors.push(err);

                    // Try to move corrupted file to DLQ
                    // Extract ULID from path for DLQ naming, or use a new ULID if that fails
                    let ulid = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .and_then(|s| Ulid::from_string(s).ok())
                        .unwrap_or_else(Ulid::new);

                    if let Err(dlq_err) = self.move_corrupted_file_to_dlq(&path, ulid) {
                        tracing::error!(
                            "Failed to move corrupted file {} to DLQ: {}",
                            path.display(),
                            dlq_err
                        );
                    }
                }
            }
        }

        // Sort by ULID (chronological order)
        entries.sort_by_key(|(ulid, _)| *ulid);

        // Report aggregate errors if any occurred
        if !errors.is_empty() {
            tracing::warn!(
                "Scan completed with {} corrupted files moved to dead letter queue",
                errors.len()
            );
            // Return error if ALL files were corrupted (nothing successfully loaded)
            if entries.is_empty() {
                return Err(S3WriteQueueError::ScanWithErrors {
                    error_count: errors.len(),
                });
            }
        }

        Ok(entries)
    }

    /// Processes a single pending file during scan
    fn scan_single_file(&self, path: &Path) -> Result<(Ulid, LayeredEvent), S3WriteQueueError> {
        // Parse ULID from filename
        let filename = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
            S3WriteQueueError::InvalidFilename {
                path: path.to_path_buf(),
            }
        })?;

        let ulid =
            Ulid::from_string(filename).map_err(|source| S3WriteQueueError::InvalidUlid {
                filename: filename.to_string(),
                source,
            })?;

        // Read and deserialize event
        let bytes = fs::read(path)?;
        let event: LayeredEvent = postcard::from_bytes(&bytes).map_err(|source| {
            S3WriteQueueError::DeserializationFailed {
                file_path: path.to_path_buf(),
                source,
            }
        })?;

        Ok((ulid, event))
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
            "Moving corrupted queue file {} to dead letter queue. Error: {}",
            ulid,
            error
        );

        // Move file to DLQ
        fs::rename(&source_path, &dest_path)?;

        Ok(())
    }

    /// Returns current queue depth (number of pending writes)
    pub fn depth(&self) -> usize {
        match self.scan() {
            Ok(items) => items.len(),
            Err(_) => 0, // Return 0 on error rather than failing
        }
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

    #[test]
    fn test_new_creates_queue_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let _queue = S3WriteQueue::new(base_path, "test_cache").unwrap();

        // Queue directory should exist
        let expected_dir = base_path.join("test_cache_s3_queue");
        assert!(expected_dir.exists());
        assert!(expected_dir.is_dir());
    }

    #[test]
    fn test_new_creates_dlq_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let _queue = S3WriteQueue::new(base_path, "test_cache").unwrap();

        // DLQ directory should exist
        let expected_dlq = base_path.join("test_cache_s3_queue").join("dead_letter");
        assert!(expected_dlq.exists());
        assert!(expected_dlq.is_dir());
    }

    #[test]
    fn test_enqueue_creates_pending_file() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

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
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

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
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        let items = queue.scan().unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_scan_returns_all_pending_files() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        let event1 = create_test_event();
        let event2 = create_test_event();
        let event3 = create_test_event();

        queue.enqueue(&event1).unwrap();
        queue.enqueue(&event2).unwrap();
        queue.enqueue(&event3).unwrap();

        let items = queue.scan().unwrap();

        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_scan_returns_ulid_sorted_order() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        let event = create_test_event();

        // Enqueue with slight delays to ensure different ULIDs
        let ulid1 = queue.enqueue(&event).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = queue.enqueue(&event).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid3 = queue.enqueue(&event).unwrap();

        let items = queue.scan().unwrap();

        // Should be in ULID order (chronological)
        assert_eq!(items[0].0, ulid1);
        assert_eq!(items[1].0, ulid2);
        assert_eq!(items[2].0, ulid3);
    }

    #[test]
    fn test_scan_ignores_non_pending_files() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        let event = create_test_event();
        queue.enqueue(&event).unwrap();

        // Create a non-pending file in the queue directory
        let queue_dir = temp_dir.path().join("test_s3_queue");
        std::fs::write(queue_dir.join("readme.txt"), "test").unwrap();
        std::fs::write(queue_dir.join("other.tmp"), "test").unwrap();

        let items = queue.scan().unwrap();

        // Should only find the one .pending file
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_remove_deletes_file() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

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
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        let ulid = Ulid::new();

        // Removing non-existent file should not error
        let result = queue.remove(ulid);
        assert!(result.is_ok());
    }

    #[test]
    fn test_move_to_dlq_moves_file() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

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
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

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
    fn test_scan_handles_corrupted_file_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        // Enqueue valid events
        let event1 = create_test_event();
        let event2 = create_test_event();
        let ulid1 = queue.enqueue(&event1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = queue.enqueue(&event2).unwrap();

        // Create a corrupted file
        let corrupted_ulid = Ulid::new();
        let corrupted_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{corrupted_ulid}.pending"));
        std::fs::write(&corrupted_path, b"corrupted data").unwrap();

        // Scan should succeed and return the valid events
        let items = queue.scan().unwrap();

        // Should have loaded the 2 valid events
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, ulid1);
        assert_eq!(items[1].0, ulid2);

        // Corrupted file should be moved to DLQ
        assert!(!corrupted_path.exists());
        let dlq_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join("dead_letter")
            .join(format!("{corrupted_ulid}.corrupted"));
        assert!(dlq_path.exists());
    }

    #[test]
    fn test_scan_with_invalid_filename() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        // Enqueue a valid event
        let event = create_test_event();
        let ulid = queue.enqueue(&event).unwrap();

        // Create a file with invalid ULID in filename
        let invalid_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join("not-a-ulid.pending");
        std::fs::write(&invalid_path, b"some data").unwrap();

        // Scan should succeed and return the valid event
        let items = queue.scan().unwrap();

        // Should have loaded the 1 valid event
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].0, ulid);

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
    fn test_scan_all_files_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        // Create multiple corrupted files
        for i in 0..3 {
            let corrupted_ulid = Ulid::new();
            let corrupted_path = temp_dir
                .path()
                .join("test_s3_queue")
                .join(format!("{corrupted_ulid}.pending"));
            std::fs::write(&corrupted_path, format!("corrupted data {i}")).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Scan should fail with ScanWithErrors since all files are corrupted
        let result = queue.scan();
        assert!(result.is_err());
        match result {
            Err(S3WriteQueueError::ScanWithErrors { error_count }) => {
                assert_eq!(error_count, 3);
            }
            _ => panic!("Expected ScanWithErrors"),
        }

        // All corrupted files should be in DLQ
        let dlq_dir = temp_dir.path().join("test_s3_queue").join("dead_letter");
        let dlq_count = std::fs::read_dir(&dlq_dir).unwrap().count();
        assert_eq!(dlq_count, 3);
    }

    #[test]
    fn test_scan_mixed_valid_and_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        // Create a mix of valid and corrupted files
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

        // Scan should succeed with a warning
        let items = queue.scan().unwrap();

        // Should have loaded the 2 valid events
        assert_eq!(items.len(), 2);

        // Verify the valid events are present
        let ulids: Vec<_> = items.iter().map(|(ulid, _)| *ulid).collect();
        assert!(ulids.contains(&ulid1));
        assert!(ulids.contains(&ulid2));

        // Corrupted file should be in DLQ
        assert!(!corrupted_path.exists());
        let dlq_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join("dead_letter")
            .join(format!("{corrupted_ulid}.corrupted"));
        assert!(dlq_path.exists());
    }

    #[test]
    fn test_depth_returns_count() {
        let temp_dir = TempDir::new().unwrap();
        let queue = S3WriteQueue::new(temp_dir.path(), "test").unwrap();

        assert_eq!(queue.depth(), 0);

        queue.enqueue(&create_test_event()).unwrap();
        assert_eq!(queue.depth(), 1);

        queue.enqueue(&create_test_event()).unwrap();
        assert_eq!(queue.depth(), 2);
    }
}
