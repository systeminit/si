use si_layer_cache::{CacheType, LayerCache};

mod disk_cache;
mod memory_cache;

#[test]
fn new() {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let _layer_cache = LayerCache::new(tempdir).expect("cannot create layer cache");
}

#[tokio::test]
async fn empty_insert_and_get() {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let layer_cache = LayerCache::new(tempdir).expect("cannot create layer cache");
    layer_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .await
        .expect("cannot insert into layer cache");

    // Confirm the insert went into the memory cache
    let memory_result = layer_cache
        .memory_cache
        .get(&CacheType::Object, b"skid row")
        .await
        .expect("cannot find value in memory cache");
    assert_eq!(b"slave to the grind", &memory_result[..]);

    // Confirm the insert went into the disk cache
    let disk_result = layer_cache
        .disk_cache
        .get(&CacheType::Object, b"skid row")
        .expect("error looking for value in disk cache")
        .expect("cannot find value in disk cache");
    assert_eq!(b"slave to the grind", &disk_result[..]);

    // Confirm we can get directly from the layer cache
    let result = layer_cache
        .get(&CacheType::Object, b"skid row")
        .await
        .expect("error finding object")
        .expect("cannot find object in cache");
    assert_eq!(b"slave to the grind", &result[..]);
}

#[tokio::test]
async fn not_in_memory_but_on_disk_insert() {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let layer_cache = LayerCache::new(tempdir).expect("cannot create layer cache");

    // Insert the object directly to disk cache
    layer_cache
        .disk_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .expect("failed to insert to disk cache");

    // There should not be anything for the key in memory cache
    assert!(!layer_cache
        .memory_cache
        .contains_key(&CacheType::Object, b"skid row"));

    // Insert through the layer cache
    layer_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .await
        .expect("cannot insert into the cache");

    // There should be an entry in memory now
    assert!(layer_cache
        .memory_cache
        .contains_key(&CacheType::Object, b"skid row"));
}

#[tokio::test]
async fn in_memory_but_not_on_disk_insert() {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let layer_cache = LayerCache::new(tempdir).expect("cannot create layer cache");

    // Insert the object directly to memory cache
    layer_cache
        .memory_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .await;

    // There should not be anything for the key in disk cache
    assert!(!layer_cache
        .disk_cache
        .contains_key(&CacheType::Object, b"skid row")
        .expect("cannot check if key exists in disk cache"));

    // Insert through the layer cache
    layer_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .await
        .expect("cannot insert into the cache");

    // There should be an entry in disk now
    assert!(layer_cache
        .disk_cache
        .contains_key(&CacheType::Object, b"skid row")
        .expect("cannot read from disk cache"));
}

#[tokio::test]
async fn get_inserts_to_memory() {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let layer_cache = LayerCache::new(tempdir).expect("cannot create layer cache");

    layer_cache
        .disk_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .expect("failed to insert to disk cache");
    assert!(!layer_cache
        .memory_cache
        .contains_key(&CacheType::Object, b"skid row"));

    layer_cache
        .get(&CacheType::Object, b"skid row")
        .await
        .expect("error getting object from cache")
        .expect("object not in cachche");

    assert!(layer_cache
        .memory_cache
        .contains_key(&CacheType::Object, b"skid row"));
}
