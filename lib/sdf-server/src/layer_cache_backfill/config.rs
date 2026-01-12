use std::time::Duration;

use chrono::{
    DateTime,
    Utc,
};

use super::error::{
    BackfillError,
    BackfillResult,
};

/// Caches that require S3 backfill for historical data migration
/// Excludes caches being migrated out of layer cache architecture (func_run, func_run_log)
const BACKFILL_CACHE_TYPES: &[&str] = &[
    "cas",
    "workspace_snapshot",
    "encrypted_secret",
    "rebase_batch",
    "change_batch",
    "split_snapshot_subgraph",
    "split_snapshot_supergraph",
    "split_snapshot_rebase_batch",
];

#[derive(Clone, Debug)]
pub struct BackfillConfig {
    pub cutoff_timestamp: DateTime<Utc>,
    pub cache_types: Vec<String>,
    pub key_batch_size: usize,
    pub checkpoint_interval: Duration,
}

impl BackfillConfig {
    pub fn from_args(
        cutoff_timestamp: Option<String>,
        cache_types: Option<String>,
        key_batch_size: usize,
        checkpoint_interval_secs: u64,
    ) -> BackfillResult<Self> {
        // Parse cutoff timestamp (required)
        let cutoff_str = cutoff_timestamp.ok_or(BackfillError::MissingCutoffTimestamp)?;
        let cutoff_timestamp = DateTime::parse_from_rfc3339(&cutoff_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Fallback: try ISO 8601 without timezone (e.g., 2025-01-14T15:30:45)
                cutoff_str.parse::<DateTime<Utc>>()
            })
            .map_err(|_| BackfillError::InvalidTimestampFormat {
                timestamp: cutoff_str,
            })?;

        // Parse cache types (required)
        let cache_types_str = cache_types.ok_or(BackfillError::NoCacheTypesSpecified)?;

        let types: Vec<String> = cache_types_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if types.is_empty() {
            return Err(BackfillError::NoCacheTypesSpecified);
        }

        // Validate all specified types are backfillable
        for cache_type in &types {
            if !BACKFILL_CACHE_TYPES.contains(&cache_type.as_str()) {
                return Err(BackfillError::CacheTypeNotBackfillable {
                    cache_type: cache_type.clone(),
                });
            }
        }

        Ok(Self {
            cutoff_timestamp,
            cache_types: types,
            key_batch_size,
            checkpoint_interval: Duration::from_secs(checkpoint_interval_secs),
        })
    }
}
