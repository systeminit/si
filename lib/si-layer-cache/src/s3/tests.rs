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
        read_retry: S3ReadRetryConfig::default(),
        num_workers: default_num_workers(),
        max_parallel_per_worker: default_max_parallel_per_worker(),
    };
    base_config.for_cache("test-cache")
}

#[tokio::test]
async fn test_transform_and_prefix_key_with_test_prefix_passthrough() {
    use tempfile::TempDir;

    let config = test_cache_config(Some("test-uuid-1234".to_string()));
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let layer = S3Layer::new(
        config,
        "test-cache",
        KeyTransformStrategy::Passthrough,
        default_num_workers(),
        default_max_parallel_per_worker(),
        S3ReadRetryConfig::default(),
        temp_dir.path(),
    )
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
    use tempfile::TempDir;

    let config = test_cache_config(None);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let layer = S3Layer::new(
        config,
        "test-cache",
        KeyTransformStrategy::Passthrough,
        default_num_workers(),
        default_max_parallel_per_worker(),
        S3ReadRetryConfig::default(),
        temp_dir.path(),
    )
    .await
    .expect("Failed to create S3Layer");

    let result = layer.transform_and_prefix_key("abc123def456");
    assert_eq!(result, "ab/c1/23/abc123def456");
}

#[tokio::test]
async fn test_transform_and_prefix_key_with_test_prefix_reverse() {
    use tempfile::TempDir;

    let config = test_cache_config(Some("test-uuid-5678".to_string()));
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let layer = S3Layer::new(
        config,
        "test-cache",
        KeyTransformStrategy::ReverseKey,
        default_num_workers(),
        default_max_parallel_per_worker(),
        S3ReadRetryConfig::default(),
        temp_dir.path(),
    )
    .await
    .expect("Failed to create S3Layer");

    // "abc123" reversed is "321cba"
    let result = layer.transform_and_prefix_key("abc123");
    assert_eq!(result, "test-uuid-5678/32/1c/ba/321cba");
}

#[tokio::test]
async fn test_transform_and_prefix_key_without_test_prefix_reverse() {
    use tempfile::TempDir;

    let config = test_cache_config(None);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let layer = S3Layer::new(
        config,
        "test-cache",
        KeyTransformStrategy::ReverseKey,
        default_num_workers(),
        default_max_parallel_per_worker(),
        S3ReadRetryConfig::default(),
        temp_dir.path(),
    )
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
        read_retry: S3ReadRetryConfig::default(),
        num_workers: default_num_workers(),
        max_parallel_per_worker: default_max_parallel_per_worker(),
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
        read_retry: S3ReadRetryConfig::default(),
        num_workers: default_num_workers(),
        max_parallel_per_worker: default_max_parallel_per_worker(),
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
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layer = S3Layer::new(
        cache_config,
        "test-cache",
        KeyTransformStrategy::Passthrough,
        default_num_workers(),
        default_max_parallel_per_worker(),
        S3ReadRetryConfig::default(),
        temp_dir.path(),
    )
    .await;
    match layer {
        Ok(_) => { /* success - we have AWS credentials available */ }
        Err(e) => {
            // Expected failure in test environment without AWS credentials
            eprintln!("Expected error without AWS credentials: {e:?}");
        }
    }
}

mod s3_error_classification_tests {
    use super::*;

    // Test the convenience predicates on S3ErrorKind
    #[test]
    fn test_error_kind_predicates() {
        let throttle = S3ErrorKind::Throttle;
        assert!(throttle.is_throttle());
        assert!(!throttle.is_serialization());
        assert!(!throttle.is_transient());

        let serialization = S3ErrorKind::Serialization;
        assert!(!serialization.is_throttle());
        assert!(serialization.is_serialization());
        assert!(!serialization.is_transient());

        let transient = S3ErrorKind::Transient;
        assert!(!transient.is_throttle());
        assert!(!transient.is_serialization());
        assert!(transient.is_transient());
    }

    // Test that Hash derivation works for S3ErrorKind
    #[test]
    fn test_error_kind_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(S3ErrorKind::Throttle);
        set.insert(S3ErrorKind::Serialization);
        set.insert(S3ErrorKind::Transient);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&S3ErrorKind::Throttle));
        assert!(set.contains(&S3ErrorKind::Serialization));
        assert!(set.contains(&S3ErrorKind::Transient));
    }

    // Test that S3ErrorKind can be cloned and compared
    #[test]
    fn test_error_kind_clone_and_eq() {
        let original = S3ErrorKind::Throttle;
        let cloned = original;
        assert_eq!(original, cloned);

        let different = S3ErrorKind::Transient;
        assert_ne!(original, different);
    }

    // Test that S3ErrorKind can be used in match expressions
    #[test]
    fn test_error_kind_match() {
        fn describe_error(kind: S3ErrorKind) -> &'static str {
            match kind {
                S3ErrorKind::Throttle => "throttle",
                S3ErrorKind::Serialization => "serialization",
                S3ErrorKind::Transient => "transient",
            }
        }

        assert_eq!(describe_error(S3ErrorKind::Throttle), "throttle");
        assert_eq!(describe_error(S3ErrorKind::Serialization), "serialization");
        assert_eq!(describe_error(S3ErrorKind::Transient), "transient");
    }

    // Documentation of classify_s3_error behavior:
    //
    // The classify_s3_error function categorizes AWS SDK errors based on:
    // 1. HTTP status codes (primary mechanism):
    //    - 503 → Throttle (with message validation as secondary check)
    //    - Other status codes → Transient
    //
    // 2. Error type patterns:
    //    - ServiceError: Check status code 503 for throttling
    //                    Also check message for SlowDown/RequestLimitExceeded/ServiceUnavailable
    //    - ConstructionFailure: → Serialization (non-retryable)
    //    - TimeoutError: → Transient (retryable)
    //    - DispatchFailure: → Transient (retryable)
    //    - ResponseError: → Transient (retryable)
    //
    // Testing this function with real AWS SDK errors requires either:
    // a) Integration tests with actual S3 backend (expensive, slow, flaky)
    // b) Mocking AWS SDK ServiceError with proper internal types (not exposed by SDK)
    //
    // The implementation follows the same pattern as categorize_error (lines 462-483)
    // which uses status code as primary classification with message as fallback.
    //
    // The straightforward pattern matching makes the implementation self-documenting,
    // and the predicates above verify the enum works correctly.
    //
    // Future work: Consider adding integration tests that trigger actual S3 errors
    // or exploring if AWS SDK test utilities become available.
}

#[test]
fn test_extract_prefix() {
    // Standard case with multiple slashes
    assert_eq!(extract_prefix("ab/cd/ef/key"), "ab/cd/ef");

    // With test prefix
    assert_eq!(
        extract_prefix("test-uuid/ab/cd/ef/key"),
        "test-uuid/ab/cd/ef"
    );

    // Many levels
    assert_eq!(extract_prefix("a/b/c/d/e/file"), "a/b/c/d/e");

    // No slashes - returns empty string
    assert_eq!(extract_prefix("no-slashes"), "");

    // Single slash
    assert_eq!(extract_prefix("prefix/key"), "prefix");

    // Empty string
    assert_eq!(extract_prefix(""), "");

    // Trailing slash
    assert_eq!(extract_prefix("prefix/"), "prefix");
}
