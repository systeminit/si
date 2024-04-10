use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::Arc;

use si_layer_cache::db::serialize;
use si_layer_cache::layer_cache::LayerCache;

async fn make_layer_cache(db_name: &str) -> LayerCache<String> {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let layer_cache = LayerCache::new("cas", tempdir.path(), super::setup_pg_db(db_name).await)
        .expect("cannot create layer cache");
    layer_cache.pg().migrate().await.expect("migrate");

    layer_cache
}

#[tokio::test]
async fn empty_insert_and_get() {
    let layer_cache = make_layer_cache("empty_insert_and_get").await;

    layer_cache
        .insert("skid row".into(), "slave to the grind".into())
        .await;

    let skid_row: Arc<str> = "skid row".into();

    // Confirm the insert went into the memory cache
    let memory_result = layer_cache
        .memory_cache()
        .get(&skid_row)
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
async fn not_in_memory_but_on_disk_insert() {
    let layer_cache = make_layer_cache("not_in_memory_but_on_disk_insert").await;

    let skid_row = "skid row";

    // Insert the object directly to disk cache
    layer_cache
        .disk_cache()
        .insert("skid row".into(), "slave to the grind".as_bytes().to_vec())
        .await
        .expect("failed to insert to disk cache");

    // There should not be anything for the key in memory cache
    assert!(!layer_cache.memory_cache().contains(skid_row));

    // Insert through the layer cache
    layer_cache
        .insert("skid row".into(), "slave to the grind".into())
        .await;

    // There should be an entry in memory now
    assert!(layer_cache.memory_cache().contains(skid_row));
}

#[tokio::test]
async fn get_inserts_to_memory() {
    let layer_cache = make_layer_cache("get_inserts_to_memory").await;

    let skid_row: Arc<str> = "skid row".into();

    let postcard_serialized = serialize::to_vec("slave to the grind").expect("should serialize");

    layer_cache
        .disk_cache()
        .insert("skid row".into(), postcard_serialized)
        .await
        .expect("failed to insert to disk cache");

    assert!(!layer_cache.memory_cache().contains(&skid_row));

    layer_cache
        .get(skid_row.clone())
        .await
        .expect("error getting object from cache")
        .expect("object not in cachche");

    assert!(layer_cache.memory_cache().contains(&skid_row));
}

#[tokio::test]
async fn get_bulk_inserts_to_memory() {
    let layer_cache = make_layer_cache("get_bulk_inserts_to_memory").await;

    let values: Vec<String> = vec![
        "skid row".into(),
        "kid scrow".into(),
        "march for macragge".into(),
    ];

    for value in &values {
        layer_cache
            .insert(value.clone().into(), value.to_string())
            .await
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
