use criterion::{black_box, criterion_group, criterion_main, Criterion};
use si_layer_cache::{CacheType, LayerCache};

use tokio::runtime;

const ASCII_LOWER: [u8; 26] = [
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p',
    b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
];

const ONE_MB: usize = 1_000_000;

pub async fn fresh_cache_count(objects: &[Vec<u8>], count: usize) {
    let tempdir = tempfile::TempDir::new_in("/home/adam/benches").expect("cannotc reate tempdir");
    let layer_cache = LayerCache::new(tempdir).expect("cannot create layer cache");
    for i in 0..count {
        layer_cache
            .insert(&CacheType::Object, [ASCII_LOWER[i]], objects[i].clone())
            .await
            .expect("cannot insert into cache");
    }
}

pub fn insert_speed_1_mb_object(c: &mut Criterion) {
    let rt = runtime::Builder::new_multi_thread()
        .build()
        .expect("cannot make tokio runtime");
    let mut objects: Vec<Vec<u8>> = Vec::with_capacity(ASCII_LOWER.len());
    for letter in ASCII_LOWER.iter() {
        let object = vec![*letter;ONE_MB];
        objects.push(object);
    }

    c.bench_function("Cold Cache insert speed 1 1mb object", |b| {
        b.to_async(&rt)
            .iter(|| fresh_cache_count(black_box(&objects[..]), 1))
    });

    c.bench_function("Cold Cache insert speed 26 1mb objects", |b| {
        b.to_async(&rt)
            .iter(|| fresh_cache_count(black_box(&objects[..]), ASCII_LOWER.len()))
    });
}

pub fn hot_read_1_mb_object(c: &mut Criterion) {
    let layer_cache = LayerCache::new("/home/adam/benches/.hot_read_1_mb_object")
        .expect("cannot create layer cache");
    let rt = runtime::Builder::new_multi_thread()
        .build()
        .expect("cannot make tokio runtime");
    let object = vec![b'a';ONE_MB];
    let _r = rt.block_on(layer_cache.insert(&CacheType::Object, b"a", object));

    c.bench_function("Hot Cache speed get one 1mb object", |b| {
        b.to_async(&rt)
            .iter(|| layer_cache.get(&CacheType::Object, [b'a']))
    });
}

pub async fn do_cold_memory_hot_disk(key: &[u8], layer_cache: &LayerCache) {
    let _r = layer_cache.get(&CacheType::Object, key).await;
    layer_cache.memory_cache.object_cache.remove(key).await;
}

pub fn hot_disk_cold_memory_read_1_mb_object(c: &mut Criterion) {
    let layer_cache = LayerCache::new("/home/adam/benches/.disk_cache_no_memory_1_mb_object")
        .expect("cannot create layer cache");
    let rt = runtime::Builder::new_multi_thread()
        .build()
        .expect("cannot make tokio runtime");
    let letter = b'a';
    let object = vec![letter;ONE_MB];
    let _r = rt.block_on(layer_cache.insert(&CacheType::Object, b"a", object));
    let key = [letter];

    c.bench_function("Hot Disk cold Memory cache speed get one 1mb object", |b| {
        b.to_async(&rt)
            .iter(|| do_cold_memory_hot_disk(black_box(&key), black_box(&layer_cache)))
    });
}

criterion_group!(
    benches,
    insert_speed_1_mb_object,
    hot_read_1_mb_object,
    hot_disk_cold_memory_read_1_mb_object
);
criterion_main!(benches);
