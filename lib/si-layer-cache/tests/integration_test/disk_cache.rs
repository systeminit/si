use std::sync::Arc;

use si_layer_cache::disk_cache::DiskCache;

#[test]
fn new() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let db = redb::Database::create(tempdir.path().join("disk-cache-test"))
        .expect("unable to open redb database");
    let _disk_cache: DiskCache =
        DiskCache::new(Arc::new(db), "random?").expect("cannot create disk cache");
}

#[tokio::test]
async fn insert_and_get() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let db = redb::Database::create(tempdir.path().join("disk-cache-test"))
        .expect("unable to open redb database");
    let disk_cache: DiskCache =
        DiskCache::new(Arc::new(db), "random?").expect("cannot create disk cache");

    disk_cache
        .insert("skid row".into(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");
    let result = disk_cache
        .get("skid row".into())
        .await
        .expect("cannot get object from disk")
        .expect("object not found in disk cache");
    assert_eq!(&result.value()[..], b"slave to the grind");
}
