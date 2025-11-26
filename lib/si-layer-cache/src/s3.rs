use std::{
    collections::HashMap,
    fmt,
    path::Path,
    sync::Arc,
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
use tokio::task::JoinHandle;

use crate::{
    error::{
        LayerDbError,
        LayerDbResult,
    },
    event::{
        LayeredEvent,
        LayeredEventPayload,
    },
    rate_limiter::RateLimitConfig,
    s3_queue_processor::S3QueueProcessor,
    s3_write_queue::S3WriteQueue,
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

/// Configuration for S3 read operation retry behavior
///
/// Controls AWS SDK retry configuration for S3Layer read operations (get, head_bucket).
/// Write operations use application-level retry via queue and have SDK retry disabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3ReadRetryConfig {
    /// Maximum number of retry attempts (default: 3)
    pub max_attempts: u32,
    /// Initial backoff delay in milliseconds (default: 100)
    pub initial_backoff_ms: u64,
    /// Maximum backoff delay in milliseconds (default: 20000)
    pub max_backoff_ms: u64,
    /// Backoff multiplier for exponential backoff (default: 2.0)
    pub backoff_multiplier: f64,
}

impl Default for S3ReadRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 20000,
            backoff_multiplier: 2.0,
        }
    }
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
    /// Rate limiting configuration for S3 writes
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// Retry configuration for S3 read operations
    #[serde(default)]
    pub read_retry: S3ReadRetryConfig,
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
            rate_limit: RateLimitConfig::default(),
            read_retry: S3ReadRetryConfig::default(),
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

/// S3-based persistence layer with internal write queue and adaptive rate limiting.
///
/// # Architecture
///
/// All writes go through a persistent disk queue (no fast path) to guarantee durability:
///
/// 1. `insert()` transforms the key and enqueues LayeredEvent to disk atomically
/// 2. Background processor dequeues in ULID order
/// 3. Processor applies backoff delay if rate limited
/// 4. Processor attempts S3 write
/// 5. On success: remove from queue, update rate limiter
/// 6. On throttle (503): increase backoff, leave in queue
/// 7. On serialization error: move to dead letter queue
/// 8. On transient error: increase backoff, leave in queue
///
/// # S3 Clients
///
/// Two independent S3 clients with different retry configurations:
///
/// - **S3Layer client**: Configurable SDK retry for resilient synchronous reads
/// - **Processor client**: SDK retry disabled (application-level retry via queue)
///
/// This separation ensures processor issues don't affect S3Layer reads, and allows
/// optimal retry strategies for each use case.
///
/// # Key Transformation
///
/// Keys are transformed at the API boundary before queueing:
///
/// - Strategy transformation (Passthrough or ReverseKey)
/// - Three-tier distribution prefix (`ab/c1/23/...`)
/// - Optional key_prefix if configured
///
/// Events in the queue contain pre-transformed keys ready for S3 storage.
/// The processor uses keys directly without transformation.
///
/// # Rate Limiting
///
/// Adaptive exponential backoff prevents constant S3 throttling:
///
/// - **On throttle:** delay *= backoff_multiplier (default: 2.0), capped at max_delay
/// - **On success:** consecutive_successes++
/// - **After N successes:** delay /= success_divisor (default: 1.5), streak resets
/// - **Below Zeno threshold (50ms):** delay resets to zero (solves dichotomy paradox)
///
/// # Configuration
///
/// Rate limiting configured via `RateLimitConfig` in `ObjectStorageConfig`:
///
/// ```json
/// {
///   "rate_limit": {
///     "min_delay_ms": 0,
///     "max_delay_ms": 5000,
///     "initial_backoff_ms": 100,
///     "backoff_multiplier": 2.0,
///     "success_divisor": null,  // defaults to multiplier * 0.75
///     "zeno_threshold_ms": 50,
///     "successes_before_reduction": 3
///   }
/// }
/// ```
///
/// # Logging
///
/// - Enqueue: TRACE level (normal operation)
/// - Success: TRACE level (expected happy path)
/// - Throttle: DEBUG level (expected, not alarming)
/// - Transient error: WARN level (unexpected but retryable)
/// - Serialization error: ERROR level (data corruption)
///
/// # Metrics
///
/// - `s3_write_queue_depth` - Current pending writes
/// - `s3_write_backoff_ms` - Current backoff delay
/// - `s3_write_attempts_total{result}` - Attempts by result (success/throttle/error)
/// - `s3_write_duration_ms` - Time from enqueue to completion
///
/// # Shutdown
///
/// On Drop:
/// 1. Signals processor to shutdown
/// 2. Processor completes in-flight write
/// 3. Processor exits (does not drain queue)
/// 4. Queue remains on disk for restart
///
/// # Startup
///
/// On initialization:
/// 1. Scans queue directory for `*.pending` files
/// 2. Loads pending writes in ULID order
/// 3. Starts processor with zero backoff
/// 4. Rate limits rediscovered naturally during processing
#[derive(Clone, Debug)]
pub struct S3Layer {
    client: Client,
    bucket_name: String,
    cache_name: String,
    strategy: KeyTransformStrategy,
    key_prefix: Option<String>,
    write_queue: Arc<S3WriteQueue>,
    processor_handle: Arc<JoinHandle<()>>,
    processor_shutdown: Arc<tokio::sync::Notify>,
}

impl S3Layer {
    /// Create a new S3Layer from configuration and strategy
    ///
    /// All writes go through a persistent queue for durability. The queue processor
    /// runs in the background and applies adaptive rate limiting if configured.
    ///
    /// # Parameters
    ///
    /// * `config` - S3 cache configuration
    /// * `cache_name` - Name of this cache (used for queue directory)
    /// * `strategy` - Key transformation strategy
    /// * `rate_limit_config` - Rate limiting configuration for adaptive backoff
    /// * `read_retry_config` - Retry configuration for S3 read operations
    /// * `queue_base_path` - Base directory for queue persistence
    pub async fn new(
        config: S3CacheConfig,
        cache_name: impl Into<String>,
        strategy: KeyTransformStrategy,
        rate_limit_config: RateLimitConfig,
        read_retry_config: S3ReadRetryConfig,
        queue_base_path: impl AsRef<Path>,
    ) -> LayerDbResult<Self> {
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
                info!(endpoint = config.endpoint, "Using S3 endpoint",);

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
        let s3_config_builder =
            aws_sdk_s3::config::Builder::from(&sdk_config).force_path_style(true);

        // Apply read retry configuration
        let retry_config = aws_sdk_s3::config::retry::RetryConfig::standard()
            .with_max_attempts(read_retry_config.max_attempts)
            .with_initial_backoff(std::time::Duration::from_millis(
                read_retry_config.initial_backoff_ms,
            ))
            .with_max_backoff(std::time::Duration::from_millis(
                read_retry_config.max_backoff_ms,
            ));

        let s3_config = s3_config_builder.retry_config(retry_config).build();

        let client = Client::from_conf(s3_config);

        // Create separate S3 client for processor with retry disabled
        // Processor uses application-level retry via queue, so SDK retry must be disabled
        let processor_s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .retry_config(aws_sdk_s3::config::retry::RetryConfig::disabled())
            .build();

        let processor_client = Client::from_conf(processor_s3_config);
        let cache_name_str = cache_name.into();
        let bucket_name = config.bucket_name;
        let key_prefix = config.key_prefix;

        // Initialize write queue
        let write_queue = S3WriteQueue::new(&queue_base_path, &cache_name_str)
            .map_err(|e| LayerDbError::S3WriteQueue(e.to_string()))?;
        let write_queue = Arc::new(write_queue);

        // Start queue processor
        // Create processor with direct S3 client access (no S3Layer dependency)
        let processor = S3QueueProcessor::new(
            Arc::clone(&write_queue),
            rate_limit_config,
            processor_client,
            bucket_name.clone(),
            cache_name_str.clone(),
        );

        let shutdown = processor.shutdown_handle();
        let handle = tokio::spawn(processor.process_queue());

        let processor_handle = Arc::new(handle);
        let processor_shutdown = shutdown;

        Ok(S3Layer {
            client,
            bucket_name,
            cache_name: cache_name_str,
            strategy,
            key_prefix,
            write_queue,
            processor_handle,
            processor_shutdown,
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
    pub async fn get(&self, key: &str) -> LayerDbResult<Option<Vec<u8>>> {
        use std::time::Instant;

        use aws_sdk_s3::{
            error::SdkError,
            operation::get_object::GetObjectError,
        };
        use telemetry_utils::histogram;

        use crate::error::AwsSdkError;

        let start = Instant::now();
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
                            "Failed to collect S3 response body for key '{key}' in cache '{}': {e}",
                            self.cache_name
                        ))
                    })?
                    .to_vec();

                histogram!(
                    layer_cache.read_latency_ms = start.elapsed().as_millis() as f64,
                    cache_name = self.cache_name.as_str(),
                    backend = "s3",
                    result = "hit"
                );

                Ok(Some(bytes))
            }
            Err(sdk_err) => {
                // Check for NoSuchKey error - return None instead of error
                if let SdkError::ServiceError(err) = &sdk_err {
                    if matches!(err.err(), GetObjectError::NoSuchKey(_)) {
                        histogram!(
                            layer_cache.read_latency_ms = start.elapsed().as_millis() as f64,
                            cache_name = self.cache_name.as_str(),
                            backend = "s3",
                            result = "miss"
                        );
                        return Ok(None);
                    }
                }

                // Also check error string for compatibility with different S3 implementations
                let error_str = sdk_err.to_string();
                if error_str.contains("NoSuchKey")
                    || error_str.contains("404")
                    || error_str.contains("Not Found")
                {
                    histogram!(
                        layer_cache.read_latency_ms = start.elapsed().as_millis() as f64,
                        cache_name = self.cache_name.as_str(),
                        backend = "s3",
                        result = "miss"
                    );
                    return Ok(None);
                }

                // Not a "not found" error - wrap with context
                let aws_error = AwsSdkError::GetObject(sdk_err);
                let s3_error = categorize_s3_error(
                    aws_error,
                    crate::error::S3Operation::Get,
                    self.cache_name.clone(),
                    key.to_string(),
                );
                Err(LayerDbError::S3(Box::new(s3_error)))
            }
        }
    }

    /// Insert an event into S3 via the write queue
    ///
    /// Transforms the key according to the configured strategy and prefix before queueing.
    /// The queued event contains the final S3 key ready to write.
    ///
    /// This is the single write interface - all S3 writes go through the queue for durability.
    pub fn insert(&self, event: &LayeredEvent) -> LayerDbResult<()> {
        // Transform key at API boundary
        let transformed_key = self.transform_and_prefix_key(&event.key);
        let transformed_key_arc: Arc<str> = Arc::from(transformed_key);

        // Create new event with transformed key
        // Most fields are Arc, so clone is cheap
        let transformed_event = LayeredEvent {
            event_id: event.event_id,
            event_kind: event.event_kind,
            key: transformed_key_arc.clone(),
            metadata: event.metadata.clone(),
            payload: LayeredEventPayload {
                db_name: event.payload.db_name.clone(),
                key: transformed_key_arc.clone(), // Reuse same Arc
                sort_key: event.payload.sort_key.clone(),
                value: event.payload.value.clone(),
            },
            web_events: event.web_events.clone(),
        };

        // Queue the transformed event
        let ulid = self
            .write_queue
            .enqueue(&transformed_event)
            .map_err(|e| LayerDbError::S3WriteQueue(e.to_string()))?;

        trace!(
            cache = %self.cache_name,
            ulid = %ulid,
            "Queued S3 write"
        );

        Ok(())
    }

    /// Get multiple values in parallel
    pub async fn get_bulk(&self, keys: &[&str]) -> LayerDbResult<HashMap<String, Vec<u8>>> {
        use futures::future::join_all;

        let futures = keys.iter().map(|&key| async move {
            let key_string = key.to_string();
            match self.get(key).await {
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
                let s3_error = categorize_s3_error(
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
}

impl Drop for S3Layer {
    fn drop(&mut self) {
        // Signal processor to shutdown
        self.processor_shutdown.notify_one();

        // Abort the processor task handle
        // Note: Can't await in Drop, processor will exit on next loop iteration
        self.processor_handle.abort();
    }
}

/// Classification of S3 errors for appropriate handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum S3ErrorKind {
    /// Throttling/rate limiting error (SlowDown, RequestLimitExceeded, ServiceUnavailable)
    Throttle,
    /// Serialization/construction error (non-retryable)
    Serialization,
    /// Transient error (retryable but not throttling)
    Transient,
}

impl S3ErrorKind {
    /// Returns true if this is a throttling error
    pub fn is_throttle(&self) -> bool {
        matches!(self, S3ErrorKind::Throttle)
    }

    /// Returns true if this is a serialization error
    pub fn is_serialization(&self) -> bool {
        matches!(self, S3ErrorKind::Serialization)
    }

    /// Returns true if this is a transient error
    pub fn is_transient(&self) -> bool {
        matches!(self, S3ErrorKind::Transient)
    }
}

/// Categorize AWS SDK error into appropriate S3Error variant with context
///
/// Used by both S3Layer (for read operations) and S3QueueProcessor (for write operations)
/// to consistently classify S3 errors for retry/backoff decisions.
pub fn categorize_s3_error(
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

/// Classify an S3 SDK error into an error kind for appropriate handling
///
/// # Classification Rules
///
/// - **Throttle**: SlowDown, RequestLimitExceeded, ServiceUnavailable (503)
/// - **Serialization**: ConstructionFailure (indicates bad request data)
/// - **Transient**: All other errors (network, timeout, internal errors, etc.)
pub fn classify_s3_error<E>(
    error: &aws_sdk_s3::error::SdkError<E, http::Response<()>>,
) -> S3ErrorKind
where
    E: std::error::Error,
{
    use aws_sdk_s3::error::SdkError;

    match error {
        SdkError::ServiceError(context) => {
            // Extract HTTP status code as primary classification mechanism
            let status = context.raw().status().as_u16();
            let error_msg = format!("{:?}", context.err());

            // Classify based on HTTP status code (503) with message validation
            if status == 503
                || error_msg.contains("SlowDown")
                || error_msg.contains("RequestLimitExceeded")
                || error_msg.contains("ServiceUnavailable")
            {
                S3ErrorKind::Throttle
            } else {
                // All other service errors are transient (InternalError, etc.)
                S3ErrorKind::Transient
            }
        }
        // AWS SDK serialization/construction errors
        SdkError::ConstructionFailure(_) => S3ErrorKind::Serialization,
        // All other errors are transient (network, timeout, etc.)
        SdkError::ResponseError(_) => S3ErrorKind::Transient,
        SdkError::TimeoutError(_) => S3ErrorKind::Transient,
        SdkError::DispatchFailure(_) => S3ErrorKind::Transient,
        _ => S3ErrorKind::Transient,
    }
}

#[cfg(test)]
mod tests;
