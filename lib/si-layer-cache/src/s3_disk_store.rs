use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
    time::Instant,
};

use telemetry::prelude::*;
use telemetry_utils::histogram;

use crate::event::{
    LayeredEvent,
    LayeredEventId,
};

#[derive(Debug, thiserror::Error)]
pub enum S3DiskStoreError {
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(String),

    #[error("Failed to serialize event: {0}")]
    Serialization(String),

    #[error("Failed to deserialize event: {0}")]
    Deserialization(String),

    #[error("Failed to write event: {0}")]
    Write(String),

    #[error("Failed to read event: {0}")]
    Read(String),

    #[error("Failed to remove event: {0}")]
    Remove(String),

    #[error("Failed to move event to dead letter queue: {0}")]
    MoveToDeadLetterQueue(String),

    #[error("Failed to scan directory: {0}")]
    Scan(String),
}

/// Minimal disk I/O abstraction for S3 event persistence
pub struct S3DiskStore {
    /// Base path for queue storage
    base_path: PathBuf,
    /// Path for dead letter queue
    dead_letter_queue_path: PathBuf,
    /// Cache name for path organization
    cache_name: String,
}

impl S3DiskStore {
    /// Create a new disk store
    pub fn new(base_path: &Path, cache_name: &str) -> Result<Self, S3DiskStoreError> {
        let queue_path = base_path.join("s3_write_queue").join(cache_name);
        let dead_letter_queue_path = base_path
            .join("s3_write_dead_letter_queue")
            .join(cache_name);

        // Create directories if they don't exist
        fs::create_dir_all(&queue_path)
            .map_err(|e| S3DiskStoreError::DirectoryCreation(e.to_string()))?;
        fs::create_dir_all(&dead_letter_queue_path)
            .map_err(|e| S3DiskStoreError::DirectoryCreation(e.to_string()))?;

        Ok(Self {
            base_path: queue_path,
            dead_letter_queue_path,
            cache_name: cache_name.to_string(),
        })
    }

    /// Write event to disk, return ULID
    pub fn write_event(&self, event: &LayeredEvent) -> Result<LayeredEventId, S3DiskStoreError> {
        let start = Instant::now();
        let event_id = event.event_id;
        let file_path = self.base_path.join(event_id.to_string());

        let serialized = serde_json::to_vec(event).map_err(|e| {
            histogram!(
                s3_disk_write_duration_ms = start.elapsed().as_millis() as f64,
                cache_name = self.cache_name.as_str(),
                backend = "s3",
                status = "serialization_error"
            );
            S3DiskStoreError::Serialization(e.to_string())
        })?;

        fs::write(&file_path, serialized).map_err(|e| {
            histogram!(
                s3_disk_write_duration_ms = start.elapsed().as_millis() as f64,
                cache_name = self.cache_name.as_str(),
                backend = "s3",
                status = "write_error"
            );
            S3DiskStoreError::Write(e.to_string())
        })?;

        histogram!(
            s3_disk_write_duration_ms = start.elapsed().as_millis() as f64,
            cache_name = self.cache_name.as_str(),
            backend = "s3",
            status = "success"
        );

        Ok(event_id)
    }

    /// Read event from disk by ULID
    pub fn read_event(&self, event_id: LayeredEventId) -> Result<LayeredEvent, S3DiskStoreError> {
        let file_path = self.base_path.join(event_id.to_string());

        let data = fs::read(&file_path).map_err(|e| S3DiskStoreError::Read(e.to_string()))?;

        let event: LayeredEvent = serde_json::from_slice(&data)
            .map_err(|e| S3DiskStoreError::Deserialization(e.to_string()))?;

        Ok(event)
    }

    /// Remove event from disk after successful upload
    pub fn remove(&self, event_id: LayeredEventId) -> Result<(), S3DiskStoreError> {
        let file_path = self.base_path.join(event_id.to_string());

        fs::remove_file(&file_path).map_err(|e| S3DiskStoreError::Remove(e.to_string()))?;

        Ok(())
    }

    /// Move event to dead letter queue
    pub fn move_to_dead_letter_queue(
        &self,
        event_id: LayeredEventId,
    ) -> Result<(), S3DiskStoreError> {
        let source_path = self.base_path.join(event_id.to_string());
        let dest_path = self.dead_letter_queue_path.join(event_id.to_string());

        fs::rename(&source_path, &dest_path)
            .map_err(|e| S3DiskStoreError::MoveToDeadLetterQueue(e.to_string()))?;

        Ok(())
    }

    /// Scan directory for existing ULIDs on startup
    pub fn scan_ulids(&self) -> Result<Vec<LayeredEventId>, S3DiskStoreError> {
        let mut event_ids = Vec::new();

        let entries =
            fs::read_dir(&self.base_path).map_err(|e| S3DiskStoreError::Scan(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| S3DiskStoreError::Scan(e.to_string()))?;

            if let Some(filename) = entry.file_name().to_str() {
                if let Ok(event_id) = filename.parse::<LayeredEventId>() {
                    event_ids.push(event_id);
                }
            }
        }

        Ok(event_ids)
    }
}
