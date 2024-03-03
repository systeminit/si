use si_layer_cache::memory_cache::MemoryCache;
use si_layer_cache::CacheType;


#[test]
fn new() {
    let _memory_cache = MemoryCache::new();
}

#[tokio::test]
async fn insert_and_get() {
    let memory_cache = MemoryCache::new();
    memory_cache
        .insert(&CacheType::Object, b"skid row", b"slave to the grind")
        .await;
    let result = memory_cache
        .get(&CacheType::Object, b"skid row")
        .await
        .expect("cannot get from memory cache");
    assert_eq!(b"slave to the grind", &result[..]);
}
