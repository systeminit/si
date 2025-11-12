use std::sync::Arc;

use rand::{
    seq::SliceRandom,
    thread_rng,
};
use si_layer_cache::{
    db::serialize,
    hybrid_cache::CacheConfig,
    layer_cache::LayerCache,
    persister::PersisterMode,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

async fn make_layer_cache(db_name: &str) -> Arc<LayerCache<String>> {
    let layer_cache = LayerCache::new(
        "cas",
        super::setup_pg_db(db_name).await,
        CacheConfig::default(),
        super::setup_compute_executor(),
        TaskTracker::new(),
        CancellationToken::new(),
        None,
        PersisterMode::PostgresOnly,
    )
    .await
    .expect("cannot create layer cache");
    layer_cache.pg().migrate().await.expect("migrate");

    layer_cache
}

#[tokio::test]
async fn empty_insert_and_get() {
    let layer_cache = make_layer_cache("empty_insert_and_get").await;

    let value = "slave to the grind";
    let size_hint = value.len();
    layer_cache.insert("skid row".into(), value.into(), size_hint);

    let skid_row: Arc<str> = "skid row".into();

    // Confirm the insert went into the memory cache
    let memory_result = layer_cache
        .cache()
        .get(skid_row.clone())
        .await
        .expect("cannot find value in memory cache");
    assert_eq!("slave to the grind", &memory_result[..]);

    // Confirm we can get directly from the layer cache
    let result = layer_cache
        .get(skid_row)
        .await
        .expect("error finding object")
        .expect("cannot find object in cache");

    assert_eq!("slave to the grind", &result[..]);
}

#[tokio::test]
async fn get_inserts_to_memory() {
    let layer_cache = make_layer_cache("get_inserts_to_memory").await;

    let skid_row: Arc<str> = "skid row".into();

    let (postcard_serialized, _) =
        serialize::to_vec("slave to the grind").expect("should serialize");

    layer_cache
        .cache()
        .insert_raw_bytes("skid row".into(), postcard_serialized);

    layer_cache
        .get(skid_row.clone())
        .await
        .expect("error getting object from cache")
        .expect("object not in cachche");

    assert!(layer_cache.cache().contains(&skid_row));
}

#[tokio::test]
async fn get_bulk_inserts() {
    let layer_cache = make_layer_cache("get_bulk_inserts_to_memory").await;

    let values: Vec<String> = vec![
        "skid row".into(),
        "kid scrow".into(),
        "march for macragge".into(),
    ];

    for value in &values {
        layer_cache.insert(value.clone().into(), value.to_string(), value.len());
    }

    let get_values = layer_cache
        .get_bulk(&values)
        .await
        .expect("should get bulk");

    // It's a little tricker than it looks to get the value out if you have
    // the original Arc<str> that you used for insertion.
    for value in values {
        let key = value.to_string();
        assert_eq!(key, get_values[&key]);
    }
}

#[tokio::test]
async fn get_bulk_from_db() {
    let layer_cache = make_layer_cache("get_bulk").await;

    let mut values: [Arc<str>; 5] = [
        "skid row".into(),
        "kid scrow".into(),
        "march for macragge".into(),
        "magnus did nothing wrong".into(),
        "steppa pig".into(),
    ];

    let mut rng = thread_rng();

    for _i in 0..5 {
        values.shuffle(&mut rng);
        for value in &values {
            let _ = layer_cache
                .pg()
                .insert(value, "cas", value.as_ref().as_bytes())
                .await;
        }

        let get_values = layer_cache
            .pg()
            .get_many(&values)
            .await
            .expect("should get bulk")
            .expect("should have results");

        for value in &values {
            assert_eq!(value.as_bytes(), get_values[&value.as_ref().to_string()]);
        }
    }
}

#[tokio::test]
async fn get_last_four_from_database() {
    let layer_cache = make_layer_cache("get_last_four_from_database").await;

    let values: [Arc<str>; 5] = [
        "skid row".into(),
        "kid scrow".into(),
        "march for macragge".into(),
        "magnus did nothing wrong".into(),
        "steppa pig".into(),
    ];

    for value in &values {
        let _ = layer_cache
            .pg()
            .insert(value, "gettin'", value.as_ref().as_bytes())
            .await;
    }

    let get_values = layer_cache
        .pg()
        .get_most_recent(4)
        .await
        .expect("should get")
        .expect("should have results");

    assert_eq!(get_values.len(), 4);
}
