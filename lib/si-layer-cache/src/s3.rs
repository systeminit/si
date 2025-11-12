use serde::{Deserialize, Serialize};

/// Strategy for transforming keys before storage in S3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyTransformStrategy {
    /// No transformation - use key as-is
    /// Use for content-addressable keys (hashes) that are already well-distributed
    Passthrough,

    /// Reverse the entire key string for even distribution
    /// Use for ULID-based keys where timestamp prefix causes hotspotting
    ReverseKey,
}

impl KeyTransformStrategy {
    /// Apply transformation to a key
    pub fn transform(&self, key: &str) -> String {
        match self {
            Self::Passthrough => key.to_string(),
            Self::ReverseKey => key.chars().rev().collect(),
        }
    }
}

impl Default for KeyTransformStrategy {
    fn default() -> Self {
        Self::Passthrough
    }
}

/// S3 authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum S3AuthConfig {
    /// Static credentials (for local dev, MinIO, etc.)
    StaticCredentials {
        access_key: String,
        secret_key: String,
    },
    /// IAM role-based authentication (for production AWS)
    /// Uses AWS SDK's default credential provider chain
    IamRole,
}

/// Resolved S3 configuration for a specific cache
#[derive(Debug, Clone)]
pub struct S3CacheConfig {
    /// S3 endpoint URL (e.g., `http://localhost:9000` or `https://s3.us-west-2.amazonaws.com`)
    pub endpoint: String,
    /// Complete bucket name for this cache
    pub bucket_name: String,
    /// AWS region (e.g., `us-east-1`)
    pub region: String,
    /// Authentication method
    pub auth: S3AuthConfig,
    /// Optional key prefix for test isolation
    pub key_prefix: Option<String>,
}

/// Configuration for S3-compatible object storage.
///
/// Supports AWS S3 and S3-compatible services (VersityGW, MinIO, etc).
///
/// Each cache gets its own bucket: `{bucket_prefix}-{cache_name}[-{bucket_suffix}]`.
/// Cache names with underscores are converted to hyphens for DNS compliance.
///
/// # Test Isolation
///
/// The `key_prefix` field provides namespace isolation for concurrent tests.
/// When set, all object keys are prefixed with this value after key transformation
/// and distribution prefixing. This mirrors the NATS `subject_prefix` pattern.
///
/// Example key transformation with prefix:
/// - Original key: `"abc123def456"`
/// - After Passthrough: `"abc123def456"`
/// - After three-tier: `"ab/c1/23/abc123def456"`
/// - After test prefix: `"test-uuid-1234/ab/c1/23/abc123def456"`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageConfig {
    /// S3 endpoint URL (e.g., `http://localhost:9000` or `https://s3.us-west-2.amazonaws.com`)
    pub endpoint: String,
    /// Bucket prefix for all caches (e.g., `si-layer-cache`)
    pub bucket_prefix: String,
    /// Optional suffix appended to final bucket name (e.g., `production`, `tools`)
    /// Final bucket: `{bucket_prefix}-{cache_name}[-{bucket_suffix}]`
    pub bucket_suffix: Option<String>,
    /// AWS region (e.g., `us-east-1`)
    pub region: String,
    /// Authentication method
    pub auth: S3AuthConfig,
    /// Optional key prefix for test isolation
    /// When set, all object keys are prefixed with this value after transformation
    pub key_prefix: Option<String>,
}

impl Default for ObjectStorageConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_string(),
            bucket_prefix: "si-layer-cache".to_string(),
            bucket_suffix: None,
            region: "us-east-1".to_string(),
            auth: S3AuthConfig::StaticCredentials {
                access_key: "siadmin".to_string(),
                secret_key: "bugbear".to_string(),
            },
            key_prefix: None,
        }
    }
}

impl ObjectStorageConfig {
    /// Create cache-specific configuration with resolved bucket name
    ///
    /// Normalizes cache name for DNS-compliant bucket naming.
    /// Final bucket name: `{bucket_prefix}-{cache_name}[-{bucket_suffix}]`
    pub fn for_cache(&self, cache_name: &str) -> S3CacheConfig {
        // S3 bucket names: lowercase, hyphens allowed, no underscores
        let normalized_name = cache_name.replace('_', "-");

        let bucket_name = match &self.bucket_suffix {
            Some(suffix) => format!("{}-{}-{}", self.bucket_prefix, normalized_name, suffix),
            None => format!("{}-{}", self.bucket_prefix, normalized_name),
        };

        S3CacheConfig {
            endpoint: self.endpoint.clone(),
            bucket_name,
            region: self.region.clone(),
            auth: self.auth.clone(),
            key_prefix: self.key_prefix.clone(),
        }
    }
}
