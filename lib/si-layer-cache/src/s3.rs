use std::{
    collections::HashMap,
    fmt,
};

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use serde::{
    Deserialize,
    Serialize,
};
use si_std::SensitiveString;
use strum::AsRefStr;
use telemetry::prelude::*;

use crate::error::{
    LayerDbError,
    LayerDbResult,
};

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
#[derive(Clone, AsRefStr, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum S3AuthConfig {
    /// Static credentials (for local dev, MinIO, etc.)
    StaticCredentials {
        access_key: SensitiveString,
        secret_key: SensitiveString,
    },
    /// IAM role-based authentication (for production AWS)
    /// Uses AWS SDK's default credential provider chain
    IamRole,
}

impl fmt::Debug for S3AuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// Resolved S3 configuration for a specific cache
#[derive(Debug, Clone)]
pub struct S3CacheConfig {
    /// S3 endpoint URL (e.g., `http://localhost:9200` or `https://s3.us-west-2.amazonaws.com`)
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
    /// S3 endpoint URL (e.g., `http://localhost:9200` or `https://s3.us-west-2.amazonaws.com`)
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
            endpoint: "http://localhost:9200".to_string(),
            bucket_prefix: "si-layer-cache".to_string(),
            bucket_suffix: None,
            region: "us-east-1".to_string(),
            auth: S3AuthConfig::StaticCredentials {
                access_key: "siadmin".into(),
                secret_key: "bugbear".into(),
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

/// S3-compatible object storage layer
#[derive(Clone, Debug)]
pub struct S3Layer {
    client: Client,
    bucket_name: String,
    strategy: KeyTransformStrategy,
    key_prefix: Option<String>,
}

impl S3Layer {
    /// Create a new S3Layer from configuration and strategy
    pub async fn new(config: S3CacheConfig, strategy: KeyTransformStrategy) -> LayerDbResult<Self> {
        info!(
            layer_db.s3.auth_mode = config.auth.as_ref(),
            layer_db.s3.bucket_name = config.bucket_name,
            "Creating S3 layer",
        );
        let sdk_config = match &config.auth {
            S3AuthConfig::StaticCredentials {
                access_key,
                secret_key,
            } => {
                // Static credential flow for dev/MinIO
                let credentials = Credentials::new(
                    access_key.as_str(),
                    secret_key.as_str(),
                    None,     // session token
                    None,     // expiration
                    "static", // provider name
                );

                aws_config::SdkConfig::builder()
                    .endpoint_url(&config.endpoint)
                    .region(Region::new(config.region.clone()))
                    .credentials_provider(
                        aws_credential_types::provider::SharedCredentialsProvider::new(credentials),
                    )
                    .behavior_version(aws_config::BehaviorVersion::latest())
                    .build()
            }
            S3AuthConfig::IamRole => {
                // Use si-aws-config which properly loads credentials, adds retry config, and validates via STS
                si_aws_config::AwsConfig::from_env()
                    .await
                    .map_err(LayerDbError::AwsConfig)?
            }
        };

        // VersityGW with POSIX backend requires path-style bucket access
        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Ok(S3Layer {
            client,
            bucket_name: config.bucket_name,
            strategy,
            key_prefix: config.key_prefix,
        })
    }

    /// Get the key transform strategy used by this S3Layer
    pub fn strategy(&self) -> KeyTransformStrategy {
        self.strategy
    }

    /// Transform and apply three-tier prefix to key, with optional test prefix
    ///
    /// Transformation order:
    /// 1. Apply strategy transformation (e.g., ReverseKey)
    /// 2. Apply three-tier distribution prefix (ab/c1/23/...)
    /// 3. Apply test key prefix if present (test-uuid/ab/c1/23/...)
    ///
    /// Example with Passthrough:
    ///   "abc123..." -> "ab/c1/23/abc123..."
    /// Example with Passthrough + test prefix:
    ///   "abc123..." -> "test-uuid-1234/ab/c1/23/abc123..."
    /// Example with ReverseKey + test prefix:
    ///   "abc123..." -> reverse -> "...321cba" -> "../.3/21/...321cba" -> "test-uuid-1234/../.3/21/...321cba"
    fn transform_and_prefix_key(&self, key: &str) -> String {
        // Step 1: Apply strategy transformation (e.g., reverse)
        let transformed = self.strategy.transform(key);

        // Step 2: Apply three-tier prefixing for S3 distribution
        // Takes first 6 chars and creates: XX/YY/ZZ/full_key
        let with_distribution_prefix = if transformed.len() >= 6 {
            format!(
                "{}/{}/{}/{}",
                &transformed[..2],
                &transformed[2..4],
                &transformed[4..6],
                transformed
            )
        } else if transformed.len() >= 4 {
            format!(
                "{}/{}/{}",
                &transformed[..2],
                &transformed[2..4],
                transformed
            )
        } else if transformed.len() >= 2 {
            format!("{}/{}", &transformed[..2], transformed)
        } else {
            transformed
        };

        // Step 3: Apply test key prefix if present (outermost prefix)
        match &self.key_prefix {
            Some(prefix) => format!("{prefix}/{with_distribution_prefix}"),
            None => with_distribution_prefix,
        }
    }

    /// Get a value by key from S3
    pub async fn get(&self, key: &str, cache_name: &str) -> LayerDbResult<Option<Vec<u8>>> {
        use aws_sdk_s3::{
            error::SdkError,
            operation::get_object::GetObjectError,
        };

        use crate::error::AwsSdkError;

        let s3_key = self.transform_and_prefix_key(key);

        match self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(s3_key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| {
                        // TODO: Body collection errors could be structured in the future
                        // For now, these are rare (occur after successful HTTP response) and
                        // ByteStreamError is not an SDK operation error, so we use a simple format
                        LayerDbError::ContentConversion(format!(
                            "Failed to collect S3 response body for key '{key}' in cache '{cache_name}': {e}"
                        ))
                    })?
                    .to_vec();

                Ok(Some(bytes))
            }
            Err(sdk_err) => {
                // Check for NoSuchKey error - return None instead of error
                if let SdkError::ServiceError(err) = &sdk_err {
                    if matches!(err.err(), GetObjectError::NoSuchKey(_)) {
                        return Ok(None);
                    }
                }

                // Also check error string for compatibility with different S3 implementations
                let error_str = sdk_err.to_string();
                if error_str.contains("NoSuchKey")
                    || error_str.contains("404")
                    || error_str.contains("Not Found")
                {
                    return Ok(None);
                }

                // Not a "not found" error - wrap with context
                let aws_error = AwsSdkError::GetObject(sdk_err);
                let s3_error = Self::categorize_error(
                    aws_error,
                    crate::error::S3Operation::Get,
                    cache_name.to_string(),
                    key.to_string(),
                );
                Err(LayerDbError::S3(Box::new(s3_error)))
            }
        }
    }

    /// Insert a value into S3
    pub async fn insert(
        &self,
        key: &str,
        _sort_key: &str,
        value: &[u8],
        cache_name: &str,
    ) -> LayerDbResult<()> {
        use crate::error::AwsSdkError;

        let s3_key = self.transform_and_prefix_key(key);

        match self
            .client
            .put_object()
            .bucket(&self.bucket_name)
            .key(s3_key)
            .body(value.to_vec().into())
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(sdk_err) => {
                let aws_error = AwsSdkError::PutObject(sdk_err);
                let s3_error = Self::categorize_error(
                    aws_error,
                    crate::error::S3Operation::Put,
                    cache_name.to_string(),
                    key.to_string(),
                );
                Err(LayerDbError::S3(Box::new(s3_error)))
            }
        }
    }

    /// Get multiple values in parallel
    pub async fn get_bulk(
        &self,
        keys: &[&str],
        cache_name: &str,
    ) -> LayerDbResult<HashMap<String, Vec<u8>>> {
        use futures::future::join_all;

        let futures = keys.iter().map(|&key| async move {
            let key_string = key.to_string();
            match self.get(key, cache_name).await {
                Ok(Some(value)) => Some((key_string, value)),
                Ok(None) => None,
                Err(_) => None, // Ignore individual errors in bulk fetch
            }
        });

        let results = join_all(futures).await;
        Ok(results.into_iter().flatten().collect())
    }

    /// Ensure bucket exists (no schema migrations needed for S3)
    pub async fn migrate(&self) -> LayerDbResult<()> {
        use crate::error::AwsSdkError;

        // Check if bucket exists - buckets should be pre-created by infrastructure
        // If bucket doesn't exist, this will return an error which should be treated as retryable
        match self
            .client
            .head_bucket()
            .bucket(&self.bucket_name)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(sdk_err) => {
                let aws_error = AwsSdkError::HeadBucket(sdk_err);
                let s3_error = Self::categorize_error(
                    aws_error,
                    crate::error::S3Operation::HeadBucket,
                    // Use bucket name as cache name for head_bucket operations
                    self.bucket_name.clone(),
                    String::new(), // No specific key for head_bucket
                );
                Err(LayerDbError::S3(Box::new(s3_error)))
            }
        }
    }

    /// Categorize AWS SDK error into appropriate S3Error variant with context
    fn categorize_error(
        aws_error: crate::error::AwsSdkError,
        operation: crate::error::S3Operation,
        cache_name: String,
        key: String,
    ) -> crate::error::S3Error {
        use crate::error::{
            AwsSdkError,
            S3Error,
        };

        // Extract message from AWS error for display
        let message = aws_error.to_string();

        // Helper to determine error category from status and message
        let determine_category = |status: u16, error_msg: &str| -> &'static str {
            if status == 403
                || error_msg.contains("AccessDenied")
                || error_msg.contains("InvalidAccessKeyId")
                || error_msg.contains("SignatureDoesNotMatch")
            {
                "authentication"
            } else if status == 404
                || error_msg.contains("NoSuchBucket")
                || error_msg.contains("NoSuchKey")
            {
                "not_found"
            } else if status == 503
                || error_msg.contains("SlowDown")
                || error_msg.contains("RequestLimitExceeded")
                || error_msg.contains("ServiceUnavailable")
            {
                "throttling"
            } else {
                "other"
            }
        };

        // Determine error category based on AWS SDK error type
        // Note: Can't use match arm `|` patterns because each variant has different inner types
        let category = match &aws_error {
            AwsSdkError::PutObject(sdk_err) => match sdk_err {
                aws_sdk_s3::error::SdkError::ServiceError(err) => {
                    let status = err.raw().status().as_u16();
                    let error_msg = format!("{:?}", err.err());
                    determine_category(status, &error_msg)
                }
                aws_sdk_s3::error::SdkError::TimeoutError(_)
                | aws_sdk_s3::error::SdkError::DispatchFailure(_)
                | aws_sdk_s3::error::SdkError::ResponseError(_) => "network",
                aws_sdk_s3::error::SdkError::ConstructionFailure(_) => "configuration",
                _ => "other",
            },
            AwsSdkError::GetObject(sdk_err) => match sdk_err {
                aws_sdk_s3::error::SdkError::ServiceError(err) => {
                    let status = err.raw().status().as_u16();
                    let error_msg = format!("{:?}", err.err());
                    determine_category(status, &error_msg)
                }
                aws_sdk_s3::error::SdkError::TimeoutError(_)
                | aws_sdk_s3::error::SdkError::DispatchFailure(_)
                | aws_sdk_s3::error::SdkError::ResponseError(_) => "network",
                aws_sdk_s3::error::SdkError::ConstructionFailure(_) => "configuration",
                _ => "other",
            },
            AwsSdkError::HeadBucket(sdk_err) => match sdk_err {
                aws_sdk_s3::error::SdkError::ServiceError(err) => {
                    let status = err.raw().status().as_u16();
                    let error_msg = format!("{:?}", err.err());
                    determine_category(status, &error_msg)
                }
                aws_sdk_s3::error::SdkError::TimeoutError(_)
                | aws_sdk_s3::error::SdkError::DispatchFailure(_)
                | aws_sdk_s3::error::SdkError::ResponseError(_) => "network",
                aws_sdk_s3::error::SdkError::ConstructionFailure(_) => "configuration",
                _ => "other",
            },
        };

        // Build appropriate S3Error variant based on category
        match category {
            "authentication" => S3Error::Authentication {
                operation,
                cache_name,
                key,
                message,
                source: aws_error,
            },
            "not_found" => S3Error::NotFound {
                operation,
                cache_name,
                key,
                message,
                source: aws_error,
            },
            "throttling" => S3Error::Throttling {
                operation,
                cache_name,
                key,
                message,
                source: aws_error,
            },
            "network" => S3Error::Network {
                operation,
                cache_name,
                key,
                message,
                source: aws_error,
            },
            "configuration" => S3Error::Configuration {
                operation,
                cache_name,
                key,
                message,
                source: aws_error,
            },
            _ => S3Error::Other {
                operation,
                cache_name,
                key,
                message,
                source: aws_error,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cache_config(key_prefix: Option<String>) -> S3CacheConfig {
        let base_config = ObjectStorageConfig {
            endpoint: "http://localhost:9200".to_string(),
            bucket_prefix: "test-bucket".to_string(),
            bucket_suffix: None,
            region: "us-east-1".to_string(),
            auth: S3AuthConfig::StaticCredentials {
                access_key: "key".into(),
                secret_key: "secret".into(),
            },
            key_prefix,
        };
        base_config.for_cache("test-cache")
    }

    #[tokio::test]
    async fn test_transform_and_prefix_key_with_test_prefix_passthrough() {
        let config = test_cache_config(Some("test-uuid-1234".to_string()));

        let layer = S3Layer::new(config, KeyTransformStrategy::Passthrough)
            .await
            .expect("Failed to create S3Layer");

        // Test with 6+ character key
        let result = layer.transform_and_prefix_key("abc123def456");
        assert_eq!(result, "test-uuid-1234/ab/c1/23/abc123def456");

        // Test with 4-5 character key
        let result = layer.transform_and_prefix_key("abcd");
        assert_eq!(result, "test-uuid-1234/ab/cd/abcd");

        // Test with 2-3 character key
        let result = layer.transform_and_prefix_key("ab");
        assert_eq!(result, "test-uuid-1234/ab/ab");
    }

    #[tokio::test]
    async fn test_transform_and_prefix_key_without_test_prefix_passthrough() {
        let config = test_cache_config(None);

        let layer = S3Layer::new(config, KeyTransformStrategy::Passthrough)
            .await
            .expect("Failed to create S3Layer");

        let result = layer.transform_and_prefix_key("abc123def456");
        assert_eq!(result, "ab/c1/23/abc123def456");
    }

    #[tokio::test]
    async fn test_transform_and_prefix_key_with_test_prefix_reverse() {
        let config = test_cache_config(Some("test-uuid-5678".to_string()));

        let layer = S3Layer::new(config, KeyTransformStrategy::ReverseKey)
            .await
            .expect("Failed to create S3Layer");

        // "abc123" reversed is "321cba"
        let result = layer.transform_and_prefix_key("abc123");
        assert_eq!(result, "test-uuid-5678/32/1c/ba/321cba");
    }

    #[tokio::test]
    async fn test_transform_and_prefix_key_without_test_prefix_reverse() {
        let config = test_cache_config(None);

        let layer = S3Layer::new(config, KeyTransformStrategy::ReverseKey)
            .await
            .expect("Failed to create S3Layer");

        // "abc123" reversed is "321cba"
        let result = layer.transform_and_prefix_key("abc123");
        assert_eq!(result, "32/1c/ba/321cba");
    }

    #[test]
    fn test_bucket_suffix_in_final_bucket_name() {
        let base_config = ObjectStorageConfig {
            endpoint: "http://localhost:9200".to_string(),
            bucket_prefix: "si-layer-cache".to_string(),
            bucket_suffix: Some("production".to_string()),
            region: "us-east-1".to_string(),
            auth: S3AuthConfig::StaticCredentials {
                access_key: "key".into(),
                secret_key: "secret".into(),
            },
            key_prefix: None,
        };

        let cache_config = base_config.for_cache("cas_objects");
        assert_eq!(
            cache_config.bucket_name,
            "si-layer-cache-cas-objects-production"
        );

        // Verify underscores are normalized to hyphens
        let cache_config2 = base_config.for_cache("some_cache");
        assert_eq!(
            cache_config2.bucket_name,
            "si-layer-cache-some-cache-production"
        );

        // Test without suffix
        let base_config_no_suffix = ObjectStorageConfig {
            bucket_suffix: None,
            ..base_config
        };
        let cache_config3 = base_config_no_suffix.for_cache("cas_objects");
        assert_eq!(cache_config3.bucket_name, "si-layer-cache-cas-objects");
    }

    #[tokio::test]
    async fn test_iam_auth_config_construction() {
        let config = ObjectStorageConfig {
            endpoint: "https://s3.us-west-2.amazonaws.com".to_string(),
            bucket_prefix: "si-layer-cache".to_string(),
            bucket_suffix: Some("production".to_string()),
            region: "us-west-2".to_string(),
            auth: S3AuthConfig::IamRole,
            key_prefix: None,
        };

        let cache_config = config.for_cache("test-cache");

        // Verify IAM auth is carried through
        assert!(matches!(cache_config.auth, S3AuthConfig::IamRole));
        assert_eq!(
            cache_config.bucket_name,
            "si-layer-cache-test-cache-production"
        );

        // Note: We can't easily test actual IAM credential resolution without AWS credentials
        // That would be an integration test requiring AWS setup
        let layer = S3Layer::new(cache_config, KeyTransformStrategy::Passthrough).await;
        assert!(layer.is_ok(), "Should create S3Layer with IAM config");
    }
}
