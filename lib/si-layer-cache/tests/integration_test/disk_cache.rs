use si_layer_cache::disk_cache::DiskCache;

#[test]
fn new() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let _disk_cache: DiskCache =
        DiskCache::new(tempdir.path(), "random?").expect("cannot create disk cache");
}

#[tokio::test]
async fn insert_and_get() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let disk_cache: DiskCache =
        DiskCache::new(tempdir.path(), "random?").expect("cannot create disk cache");

    disk_cache
        .insert("skid row".into(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");
    let result = disk_cache
        .get("skid row".into())
        .await
        .expect("cannot get object from disk");
    assert_eq!(&result[..], b"slave to the grind");
}

#[tokio::test]
async fn insert_and_remove() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let disk_cache: DiskCache =
        DiskCache::new(tempdir.path(), "random?").expect("cannot create disk cache");

    disk_cache
        .insert("skid row".into(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");
    disk_cache
        .get("skid row".into())
        .await
        .expect("cannot get object from disk");
    disk_cache
        .remove("skid row".into())
        .await
        .expect("cannot remove object from disk");
}

#[tokio::test]
async fn remove_never_inserted_object() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let disk_cache: DiskCache =
        DiskCache::new(tempdir.path(), "random?").expect("cannot create disk cache");
    disk_cache
        .remove("skid row".into())
        .await
        .expect("cannot remove object from disk");
}
