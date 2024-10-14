use std::time::Duration;
use tokio_util::sync::CancellationToken;

use si_layer_cache::disk_cache::{DiskCache, DiskCacheCleanupMode, DiskCacheConfig};

#[tokio::test]
async fn new() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let _disk_cache: DiskCache = DiskCache::new(DiskCacheConfig::new(
        DiskCacheCleanupMode::NoOp,
        tempdir.path(),
        "random?",
        Duration::from_secs(600),
        Duration::from_secs(600),
    ))
    .expect("cannot create disk cache");
}

#[tokio::test]
async fn insert_and_get() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let disk_cache: DiskCache = DiskCache::new(DiskCacheConfig::new(
        DiskCacheCleanupMode::NoOp,
        tempdir.path(),
        "random?",
        Duration::from_secs(600),
        Duration::from_secs(600),
    ))
    .expect("cannot create disk cache");

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
    let disk_cache: DiskCache = DiskCache::new(DiskCacheConfig::new(
        DiskCacheCleanupMode::NoOp,
        tempdir.path(),
        "random?",
        Duration::from_secs(600),
        Duration::from_secs(600),
    ))
    .expect("cannot create disk cache");

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
    let disk_cache: DiskCache = DiskCache::new(DiskCacheConfig::new(
        DiskCacheCleanupMode::NoOp,
        tempdir.path(),
        "random?",
        Duration::from_secs(600),
        Duration::from_secs(600),
    ))
    .expect("cannot create disk cache");
    disk_cache
        .remove("skid row".into())
        .await
        .expect("cannot remove object from disk");
}

#[tokio::test]
async fn remove_ttld_item() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let disk_cache: DiskCache = DiskCache::new(DiskCacheConfig::new(
        DiskCacheCleanupMode::Remove,
        tempdir.path(),
        "random?",
        Duration::from_secs(1),
        Duration::from_secs(1),
    ))
    .expect("cannot create disk cache");
    let token = CancellationToken::new();
    disk_cache.start_cleanup_task(token.clone());

    disk_cache
        .insert("skid row".into(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");

    tokio::time::sleep(Duration::from_secs(5)).await;
    let item = disk_cache.get("skid row".into()).await;
    // we should not be able to get the item as it will be cleaned up
    assert!(item.is_err());
}

#[tokio::test]
async fn noop_ttld_item() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let disk_cache: DiskCache = DiskCache::new(DiskCacheConfig::new(
        DiskCacheCleanupMode::NoOp,
        tempdir.path(),
        "random?",
        Duration::from_secs(1),
        Duration::from_secs(1),
    ))
    .expect("cannot create disk cache");
    let token = CancellationToken::new();
    disk_cache.start_cleanup_task(token.clone());

    disk_cache
        .insert("skid row".into(), b"slave to the grind".to_vec())
        .await
        .expect("cannot insert object");

    tokio::time::sleep(Duration::from_secs(5)).await;
    let item = disk_cache.get("skid row".into()).await;
    // we should not be able to get the item as it will be cleaned up
    assert!(item.is_ok());
}
