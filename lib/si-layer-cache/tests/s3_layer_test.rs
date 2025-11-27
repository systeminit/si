use si_layer_cache::ObjectStorageConfig;

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
