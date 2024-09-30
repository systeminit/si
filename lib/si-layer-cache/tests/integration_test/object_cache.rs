use si_layer_cache::object_cache::{ObjectCache, ObjectCacheConfig};

const LOCALSTACK_ENDPOINT: &str = "http://0.0.0.0:4566";

#[tokio::test]
async fn new() {
    let _object_cache: ObjectCache = ObjectCache::new(
        ObjectCacheConfig::default().with_endpoint(LOCALSTACK_ENDPOINT.to_string()),
    )
    .await
    .expect("cannot create object cache");
}

#[tokio::test]
async fn insert_and_get() {
    let object_cache: ObjectCache = ObjectCache::new(
        ObjectCacheConfig::default().with_endpoint(LOCALSTACK_ENDPOINT.to_string()),
    )
    .await
    .expect("cannot create object cache");

    object_cache
        .insert("skid row".into(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");
    let exists = object_cache
        .contains_key("skid row".into())
        .await
        .expect("cannot get object from object cache");

    assert!(exists);

    let result = object_cache
        .get("skid row".into())
        .await
        .expect("cannot get object from object cache")
        .expect("object is none when it should be Some");

    assert_eq!(&result, b"slave to the grind");
}

#[tokio::test]
async fn insert_and_remove() {
    let object_cache: ObjectCache = ObjectCache::new(
        ObjectCacheConfig::default().with_endpoint(LOCALSTACK_ENDPOINT.to_string()),
    )
    .await
    .expect("cannot create object cache");

    object_cache
        .insert("skid row".into(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");
    let result = object_cache
        .get("skid row".into())
        .await
        .expect("cannot get object from object cache")
        .expect("object is none when it should be Some");

    assert_eq!(&result, b"slave to the grind");

    object_cache
        .remove("skid row".into())
        .await
        .expect("cannot remove object from object cache");
    let result = object_cache
        .get("skid row".into())
        .await
        .expect("cannot get object from object cache");

    assert!(result.is_none());
}
