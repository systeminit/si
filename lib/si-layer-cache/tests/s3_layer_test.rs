use si_layer_cache::{
    KeyTransformStrategy,
    ObjectStorageConfig,
    S3Layer,
};

#[tokio::test]
#[ignore = "requires VersityGW running"]
async fn test_s3_put_get() {
    let config = ObjectStorageConfig::default().for_cache("test-cache");
    let s3 = S3Layer::new(config, "test-cache", KeyTransformStrategy::Passthrough)
        .await
        .expect("Failed to create S3Layer");

    // Ensure bucket exists
    s3.migrate().await.expect("Failed to create bucket");

    // Test data
    let key = "test-key-12345";
    let value = b"test value data";

    // Write
    s3.insert(key, value).await.expect("Failed to insert");

    // Read back
    let result = s3.get(key).await.expect("Failed to get");
    assert_eq!(result, Some(value.to_vec()));

    // Non-existent key
    let missing = s3.get("nonexistent").await.expect("Failed to get");
    assert_eq!(missing, None);
}

#[tokio::test]
#[ignore = "requires VersityGW running"]
async fn test_key_transform_passthrough() {
    let config = ObjectStorageConfig::default().for_cache("test-transform");
    let s3 = S3Layer::new(config, "test-transform", KeyTransformStrategy::Passthrough)
        .await
        .expect("Failed to create S3Layer");
    s3.migrate().await.expect("Failed to create bucket");

    // Content-addressable key (already well-distributed)
    let key = "abcdef123456789";

    s3.insert(key, b"value").await.expect("Failed to insert");

    // Verify we can read it back (transform is applied consistently)
    let val = s3.get(key).await.expect("Failed to get");
    assert_eq!(val, Some(b"value".to_vec()));
}

#[tokio::test]
#[ignore = "requires VersityGW running"]
async fn test_key_transform_reverse() {
    let config = ObjectStorageConfig::default().for_cache("test-reverse");
    let s3 = S3Layer::new(config, "test-reverse", KeyTransformStrategy::ReverseKey)
        .await
        .expect("Failed to create S3Layer");
    s3.migrate().await.expect("Failed to create bucket");

    // ULID-based key (timestamp prefix needs reversal)
    let key = "01HN6Y8K9Z123456789ABC";

    s3.insert(key, b"value").await.expect("Failed to insert");

    // Verify we can read it back (transform is applied consistently)
    let val = s3.get(key).await.expect("Failed to get");
    assert_eq!(val, Some(b"value".to_vec()));
}

#[tokio::test]
#[ignore = "requires VersityGW running"]
async fn test_s3_three_tier_prefix() {
    let config = ObjectStorageConfig::default().for_cache("test-prefix");
    let s3 = S3Layer::new(config, "test-prefix", KeyTransformStrategy::Passthrough)
        .await
        .expect("Failed to create S3Layer");
    s3.migrate().await.expect("Failed to create bucket");

    // Keys that should have three-tier prefixing
    let key1 = "abcdef123456";
    let key2 = "xyz789abcdef";

    s3.insert(key1, b"value1")
        .await
        .expect("Failed to insert key1");
    s3.insert(key2, b"value2")
        .await
        .expect("Failed to insert key2");

    // Verify we can read them back
    let val1 = s3.get(key1).await.expect("Failed to get key1");
    assert_eq!(val1, Some(b"value1".to_vec()));

    let val2 = s3.get(key2).await.expect("Failed to get key2");
    assert_eq!(val2, Some(b"value2".to_vec()));
}

#[tokio::test]
#[ignore = "requires VersityGW running"]
async fn test_s3_get_bulk() {
    let config = ObjectStorageConfig::default().for_cache("test-bulk");
    let s3 = S3Layer::new(config, "test-bulk", KeyTransformStrategy::Passthrough)
        .await
        .expect("Failed to create S3Layer");
    s3.migrate().await.expect("Failed to create bucket");

    // Insert multiple keys
    let keys = vec!["bulk1", "bulk2", "bulk3"];
    for key in &keys {
        s3.insert(key, key.as_bytes())
            .await
            .expect("Failed to insert");
    }

    // Bulk fetch
    let results = s3.get_bulk(&keys).await.expect("Failed to get bulk");

    assert_eq!(results.len(), 3);
    assert_eq!(results.get("bulk1"), Some(&b"bulk1".to_vec()));
    assert_eq!(results.get("bulk2"), Some(&b"bulk2".to_vec()));
    assert_eq!(results.get("bulk3"), Some(&b"bulk3".to_vec()));
}

#[test]
fn test_object_storage_config_for_cache() {
    let base_config = ObjectStorageConfig::default();

    // Test underscore to hyphen conversion
    let cas_config = base_config.for_cache("cas");
    assert_eq!(cas_config.bucket_name, "si-layer-cache-cas");

    let change_batch_config = base_config.for_cache("change_batch");
    assert_eq!(
        change_batch_config.bucket_name,
        "si-layer-cache-change-batch"
    );

    let snapshot_graph_config = base_config.for_cache("snapshot_graph");
    assert_eq!(
        snapshot_graph_config.bucket_name,
        "si-layer-cache-snapshot-graph"
    );
}
