use std::time::Duration;

use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgPoolConfig;
use telemetry::prelude::*;

/// Minimum allowed grace period (30 seconds)
const MIN_GRACE_PERIOD_SECONDS: i64 = 30;

/// Minimum allowed poll interval (30 seconds)
const MIN_POLL_INTERVAL_SECONDS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEvictionConfig {
    /// How long a snapshot must be unused before eviction (seconds)
    /// Default: 300 (5 minutes)
    /// Minimum: 30 seconds
    /// Maximum: i64::MAX
    #[serde(default = "default_grace_period")]
    pub grace_period_seconds: i64,

    /// How often to poll for eviction candidates (seconds)
    /// Default: 60 (1 minute)
    /// Minimum: 30 seconds
    /// Maximum: u64::MAX
    #[serde(default = "default_poll_interval")]
    pub poll_interval_seconds: u64,

    /// Maximum snapshots to process per query batch
    /// Default: 100
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Si-db database connection configuration
    #[serde(default)]
    pub si_db: PgPoolConfig,

    /// Layer-cache database connection configuration
    #[serde(default)]
    pub layer_cache_pg: PgPoolConfig,
}

fn default_grace_period() -> i64 {
    300
}

fn default_poll_interval() -> u64 {
    60
}

fn default_batch_size() -> usize {
    100
}

impl Default for SnapshotEvictionConfig {
    fn default() -> Self {
        Self {
            grace_period_seconds: default_grace_period(),
            poll_interval_seconds: default_poll_interval(),
            batch_size: default_batch_size(),
            si_db: PgPoolConfig::default(),
            layer_cache_pg: PgPoolConfig::default(),
        }
    }
}

impl SnapshotEvictionConfig {
    /// Validate and clamp configuration parameters
    ///
    /// Ensures grace period and poll interval are at least 30 seconds to prevent
    /// excessive database load and maintain safety margin for race condition elimination.
    ///
    /// Values below minimum are clamped and a warning is logged.
    pub fn validate_and_clamp(&mut self) {
        if self.grace_period_seconds < MIN_GRACE_PERIOD_SECONDS {
            warn!(
                configured = self.grace_period_seconds,
                minimum = MIN_GRACE_PERIOD_SECONDS,
                "grace_period_seconds below minimum, clamping to minimum"
            );
            self.grace_period_seconds = MIN_GRACE_PERIOD_SECONDS;
        }

        if self.poll_interval_seconds < MIN_POLL_INTERVAL_SECONDS {
            warn!(
                configured = self.poll_interval_seconds,
                minimum = MIN_POLL_INTERVAL_SECONDS,
                "poll_interval_seconds below minimum, clamping to minimum"
            );
            self.poll_interval_seconds = MIN_POLL_INTERVAL_SECONDS;
        }
    }

    pub fn grace_period(&self) -> Duration {
        Duration::from_secs(self.grace_period_seconds as u64)
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_interval_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SnapshotEvictionConfig::default();
        assert_eq!(config.grace_period_seconds, 300);
        assert_eq!(config.poll_interval_seconds, 60);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_grace_period_duration() {
        let config = SnapshotEvictionConfig {
            grace_period_seconds: 600,
            ..Default::default()
        };
        assert_eq!(config.grace_period().as_secs(), 600);
    }

    #[test]
    fn test_validate_and_clamp_grace_period_too_small() {
        let mut config = SnapshotEvictionConfig {
            grace_period_seconds: 29,
            ..Default::default()
        };
        config.validate_and_clamp();
        assert_eq!(config.grace_period_seconds, 30);
    }

    #[test]
    fn test_validate_and_clamp_poll_interval_too_small() {
        let mut config = SnapshotEvictionConfig {
            poll_interval_seconds: 15,
            ..Default::default()
        };
        config.validate_and_clamp();
        assert_eq!(config.poll_interval_seconds, 30);
    }

    #[test]
    fn test_validate_and_clamp_minimum_values_unchanged() {
        let mut config = SnapshotEvictionConfig {
            grace_period_seconds: 30,
            poll_interval_seconds: 30,
            batch_size: 1,
            ..Default::default()
        };
        config.validate_and_clamp();
        assert_eq!(config.grace_period_seconds, 30);
        assert_eq!(config.poll_interval_seconds, 30);
    }

    #[test]
    fn test_poll_interval_duration() {
        let config = SnapshotEvictionConfig {
            poll_interval_seconds: 120,
            ..Default::default()
        };
        assert_eq!(config.poll_interval().as_secs(), 120);
    }
}
