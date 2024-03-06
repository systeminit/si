use criterion::{black_box, criterion_group, criterion_main, Criterion};
use moka::future::Cache;
use si_layer_cache::LayerCache;

use tokio::runtime;

const ASCII_LOWER: [u8; 26] = [
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p',
    b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
];

const ONE_MB: usize = 1_000_000;

type BenchmarkCache = Cache<[u8; 1], Vec<u8>>;

pub async fn fresh_cache_count(objects: &[Vec<u8>], count: usize) {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");

    let db = sled::open(&tempdir).expect("unable to create sled database");

    let cache: BenchmarkCache = Cache::new(10_000);

    let layer_cache: LayerCache<[u8; 1], Vec<u8>> =
        LayerCache::new(db, "temp", Box::new(cache)).expect("cannot create layer cache");
    for i in 0..count {
        layer_cache
            .insert([ASCII_LOWER[i]], objects[i].clone())
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
        let object = vec![*letter; ONE_MB];
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
    let target_dir = tempfile::tempdir().expect("cannot create temp dir");
    let target_path = target_dir
        .path()
        .as_os_str()
        .to_os_string()
        .into_string()
        .expect("get string of temp dir");
    let path = format!("{target_path}/.hot_read_1_mb_object");

    let db = sled::open(path).expect("unable to create sled database");
    let cache: BenchmarkCache = Cache::new(10_000);

    let layer_cache: LayerCache<[u8; 1], Vec<u8>> =
        LayerCache::new(db, "hot_read_1_mb", Box::new(cache)).expect("cannot create layer cache");

    let rt = runtime::Builder::new_multi_thread()
        .build()
        .expect("cannot make tokio runtime");
    let letter = b'a';
    let object = vec![letter; ONE_MB];
    let _r = rt.block_on(layer_cache.insert([letter], object));

    c.bench_function("Hot Cache speed get one 1mb object", |b| {
        b.to_async(&rt).iter(|| layer_cache.get(&[b'a']))
    });
}

pub async fn do_cold_memory_hot_disk(key: [u8; 1], layer_cache: &LayerCache<[u8; 1], Vec<u8>>) {
    let _r = layer_cache.get(&key).await;
    layer_cache.remove_from_memory(key).await;
}

pub fn hot_disk_cold_memory_read_1_mb_object(c: &mut Criterion) {
    let target_dir = tempfile::tempdir().expect("cannot create temp dir");
    let target_path = target_dir
        .path()
        .as_os_str()
        .to_os_string()
        .into_string()
        .expect("get string of temp dir");
    let path = format!("{target_path}/.disk_cache_no_memory_1_mb_object");

    let db = sled::open(path).expect("unable to create sled database");
    let cache: BenchmarkCache = Cache::new(10_000);

    let layer_cache =
        LayerCache::new(db, "foo", Box::new(cache)).expect("cannot create layer cache");
    let rt = runtime::Builder::new_multi_thread()
        .build()
        .expect("cannot make tokio runtime");
    let letter = b'a';
    let object = vec![letter; ONE_MB];
    let _r = rt.block_on(layer_cache.insert([letter], object));
    let key = [letter];

    c.bench_function("Hot Disk cold Memory cache speed get one 1mb object", |b| {
        b.to_async(&rt)
            .iter(|| do_cold_memory_hot_disk(black_box(key), black_box(&layer_cache)))
    });
}

criterion_group!(
    benches,
    insert_speed_1_mb_object,
    hot_read_1_mb_object,
    hot_disk_cold_memory_read_1_mb_object
);
criterion_main!(benches);
