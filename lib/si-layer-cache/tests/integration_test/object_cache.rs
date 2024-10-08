use std::sync::Arc;

use si_layer_cache::object_cache::ObjectCache;

use crate::integration_test::setup_object_cache_config;

#[tokio::test]
async fn new() {
    let _object_cache: ObjectCache = ObjectCache::new(setup_object_cache_config().await)
        .await
        .expect("cannot create object cache");
}

#[tokio::test]
async fn insert_and_get() {
    let object_cache: ObjectCache = ObjectCache::new(setup_object_cache_config().await)
        .await
        .expect("cannot create object cache");

    let key: Arc<str> = "skid row".into();

    object_cache
        .insert(key.clone(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");

    let exists = object_cache
        .contains_key(key.clone())
        .await
        .expect("cannot get object from object cache");

    assert!(exists);

    let result = object_cache
        .get(key)
        .await
        .expect("cannot get object from object cache")
        .expect("result should be Some");

    assert_eq!(&result, b"slave to the grind");
}

#[tokio::test]
async fn insert_and_remove() {
    let object_cache: ObjectCache = ObjectCache::new(setup_object_cache_config().await)
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
