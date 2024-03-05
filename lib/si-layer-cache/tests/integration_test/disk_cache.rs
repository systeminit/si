use si_layer_cache::{disk_cache::DiskCache, CacheType};

#[test]
fn new() {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let _disk_cache =
        DiskCache::new(tempdir).expect("cannot create disk cache and a tree for each type");
}

#[tokio::test]
async fn insert_and_get() {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let disk_cache =
        DiskCache::new(tempdir).expect("cannot create disk cache and a tree for each type");
    disk_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .expect("cannot insert object");
    let result = disk_cache
        .get(&CacheType::Object, b"skid row")
        .expect("cannot get object from disk")
        .expect("object not found in disk cache");
    assert_eq!(&result[..], b"slave to the grind");
}
