use buck2_resources::Buck2Resources;
use moka::future::Cache;
use si_data_pg::{PgPool, PgPoolConfig};
use si_layer_cache::LayerCache;
use std::env;
use std::path::Path;
use tokio::task::JoinSet;

mod disk_cache;

async fn setup_pg_db(db_name: &str) -> PgPool {
    let si_pg_pool = PgPoolConfig {
        application_name: "si-layer-cache-db-tests".into(),
        certificate_path: Some(
            detect_and_configure_development()
                .try_into()
                .expect("should get a certifcate cache"),
        ),
        ..Default::default()
    };

    let test_pg_pool_config = PgPoolConfig {
        dbname: db_name.into(),
        application_name: "si-layer-cache-db-tests".into(),
        certificate_path: Some(
            detect_and_configure_development()
                .try_into()
                .expect("should get a certifcate cache"),
        ),
        ..Default::default()
    };

    let si_pg_pool = PgPool::new(&si_pg_pool)
        .await
        .expect("cannot create pg pool for tests");

    let db_drop_query = format!("DROP DATABASE IF EXISTS {}", test_pg_pool_config.dbname);

    let db_create_query = format!(
        "CREATE DATABASE {} OWNER {}",
        test_pg_pool_config.dbname, test_pg_pool_config.user
    );

    let client = si_pg_pool
        .get()
        .await
        .expect("unable to get pg_pool client");

    client
        .execute(&db_drop_query, &[])
        .await
        .expect("able to drop database for tests");

    client
        .execute(&db_create_query, &[])
        .await
        .expect("able to create database for tests");

    PgPool::new(&test_pg_pool_config)
        .await
        .expect("cannot create pg pool for tests")
}

async fn make_layer_cache(db_name: &str) -> LayerCache<&'static str, String> {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let db = sled::open(tempdir).expect("unable to open sled database");
    let cache: Cache<&'static str, String> = Cache::new(10_000);

    LayerCache::new("test1", Box::new(cache), db, setup_pg_db(db_name).await)
        .await
        .expect("cannot create layer cache")
}

#[tokio::test]
async fn empty_insert_and_get() {
    let layer_cache = make_layer_cache("empty_insert_and_get").await;

    layer_cache
        .insert("skid row", "slave to the grind".into())
        .await
        .expect("cannot insert into layer cache");
    layer_cache.join_all_write_tasks().await;

    let skid_row = "skid row";

    // Confirm the insert went into the memory cache
    let memory_result = layer_cache
        .memory_cache()
        .get_value(&skid_row)
        .await
        .expect("cannot find value in memory cache");
    assert_eq!("slave to the grind", &memory_result[..]);

    // Confirm the insert went into the disk cache
    let disk_result = layer_cache
        .disk_cache()
        .get(&skid_row)
        .expect("error looking for value in disk cache")
        .expect("cannot find value in disk cache");
    let deserialized_string: String =
        postcard::from_bytes(&disk_result).expect("should get the string");

    assert_eq!("slave to the grind", deserialized_string.as_str());

    // Confirm we can get directly from the layer cache
    let result = layer_cache
        .get(&skid_row)
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
        .insert("skid row", "slave to the grind".as_bytes())
        .expect("failed to insert to disk cache");
    layer_cache.join_all_write_tasks().await;

    // There should not be anything for the key in memory cache
    assert!(!layer_cache.memory_cache().has_key(&skid_row));

    // Insert through the layer cache
    layer_cache
        .insert("skid row", "slave to the grind".into())
        .await
        .expect("cannot insert into the cache");
    layer_cache.join_all_write_tasks().await;

    // There should be an entry in memory now
    assert!(layer_cache.memory_cache().has_key(&skid_row));
}

#[tokio::test]
async fn in_memory_but_not_on_disk_insert() {
    let layer_cache = make_layer_cache("in_memory_but_not_on_disk_insert").await;

    let skid_row = "skid row";

    // Insert the object directly to memory cache
    layer_cache
        .memory_cache()
        .insert_value("skid row", "slave to the grind".into())
        .await;

    // There should not be anything for the key in disk cache
    assert!(!layer_cache
        .disk_cache()
        .contains_key(&skid_row)
        .expect("cannot check if key exists in disk cache"));

    // Insert through the layer cache
    layer_cache
        .insert("skid row", "slave to the grind".into())
        .await
        .expect("cannot insert into the cache");
    layer_cache.join_all_write_tasks().await;

    // There should be an entry in disk now
    assert!(layer_cache
        .disk_cache()
        .contains_key(&skid_row)
        .expect("cannot read from disk cache"));
}

#[tokio::test]
async fn get_inserts_to_memory() {
    let layer_cache = make_layer_cache("get_inserts_to_memory").await;

    let skid_row = "skid row";

    let postcard_serialized = postcard::to_stdvec("slave to the grind").expect("should serialize");

    layer_cache
        .disk_cache()
        .insert("skid row", &postcard_serialized)
        .expect("failed to insert to disk cache");
    layer_cache.join_all_write_tasks().await;

    assert!(!layer_cache.memory_cache().has_key(&skid_row));

    layer_cache
        .get(&skid_row)
        .await
        .expect("error getting object from cache")
        .expect("object not in cachche");

    assert!(layer_cache.memory_cache().has_key(&skid_row));
}

#[tokio::test]
async fn multiple_mokas_single_sled() {
    let count = 10_000;
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let db = sled::open(tempdir).expect("unable to open sled database");
    let cache_even: Cache<[u8; 8], String> = Cache::new(count);

    let even_tree_name = "even_numbers";
    let odd_tree_name = "odd_numbers";
    let pg_pool = setup_pg_db("multiple_mokas_single_sled").await;

    let layer_cache_even: LayerCache<[u8; 8], String> = LayerCache::new(
        even_tree_name,
        Box::new(cache_even),
        db.clone(),
        pg_pool.clone(),
    )
    .await
    .expect("cannot create layer cache");

    let cache_odd: Cache<[u8; 8], String> = Cache::new(count);
    let layer_cache_odd = LayerCache::new(odd_tree_name, Box::new(cache_odd), db.clone(), pg_pool)
        .await
        .expect("cannot create layer cache");

    let tree_names: Vec<String> = db
        .tree_names()
        .into_iter()
        .filter_map(|name| {
            std::str::from_utf8(name.as_ref())
                .ok()
                .map(ToOwned::to_owned)
        })
        .collect();

    // Confirm that the original database, after the clones, has the trees
    assert!(tree_names.contains(&even_tree_name.to_string()));
    assert!(tree_names.contains(&odd_tree_name.to_string()));

    fn make_u64_kv(integer: u64) -> ([u8; 8], String) {
        (integer.to_le_bytes(), integer.to_string())
    }

    let mut task_set = JoinSet::new();

    let layer_cache_even_clone = layer_cache_even.clone();
    task_set.spawn(async move {
        for i in 0..(count * 2) {
            if i % 2 == 0 {
                let (key, value) = make_u64_kv(i);
                layer_cache_even_clone
                    .insert(key, value)
                    .await
                    .expect("unable to insert");
            }
        }
    });

    let layer_cache_odd_clone = layer_cache_odd.clone();
    task_set.spawn(async move {
        for i in 0..(count * 2) {
            if i % 2 != 0 {
                let (key, value) = make_u64_kv(i);
                layer_cache_odd_clone
                    .insert(key, value)
                    .await
                    .expect("unable to insert");
            }
        }
    });

    while (task_set.join_next().await).is_some() {}
    layer_cache_even.join_all_write_tasks().await;
    layer_cache_odd.join_all_write_tasks().await;

    let even_tree = db
        .open_tree(even_tree_name.as_bytes())
        .expect("unable to open even tree");
    let odd_tree = db
        .open_tree(odd_tree_name.as_bytes())
        .expect("unable to open odd tree");

    for i in 0..(count * 2) {
        let (key, value) = make_u64_kv(i);
        let even_tree_value: Option<String> = even_tree
            .get(key)
            .expect("able to get even value")
            .and_then(|value| postcard::from_bytes(value.as_ref()).ok());

        let odd_tree_value: Option<String> = odd_tree
            .get(key)
            .expect("able to get odd key value")
            .and_then(|value| postcard::from_bytes(value.as_ref()).ok());

        if i % 2 == 0 {
            assert_eq!(Some(value), even_tree_value);
            assert_eq!(None, odd_tree_value);
        } else {
            assert_eq!(Some(value), odd_tree_value);
            assert_eq!(None, even_tree_value);
        }
    }
}

/// This function is used to determine the development environment and update the [`ConfigFile`]
/// accordingly.
#[allow(clippy::disallowed_methods)]
pub fn detect_and_configure_development() -> String {
    if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        buck2_development()
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        cargo_development(dir)
    } else {
        "".to_string()
    }
}

fn buck2_development() -> String {
    let resources = Buck2Resources::read().expect("should be able to read buck2 resources");

    resources
        .get_ends_with("dev.postgres.root.crt")
        .expect("should be able to get cert")
        .to_string_lossy()
        .to_string()
}

fn cargo_development(dir: String) -> String {
    Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string()
}
