use moka::future::Cache;
use si_layer_cache::LayerCache;
use tokio::task::JoinSet;

mod disk_cache;

fn make_layer_cache() -> LayerCache<&'static str, String> {
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let db = sled::open(tempdir).expect("unable to open sled database");
    let cache: Cache<&'static str, String> = Cache::new(10_000);

    let layer_cache: LayerCache<&'static str, String> =
        LayerCache::new(db, "test1", Box::new(cache)).expect("cannot create layer cache");

    layer_cache
}

#[tokio::test]
async fn empty_insert_and_get() {
    let layer_cache = make_layer_cache();

    layer_cache
        .insert("skid row", "slave to the grind".into())
        .await
        .expect("cannot insert into layer cache");

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
        .disk_cache
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
    let layer_cache = make_layer_cache();

    let skid_row = "skid row";

    // Insert the object directly to disk cache
    layer_cache
        .disk_cache
        .insert("skid row", "slave to the grind".as_bytes())
        .expect("failed to insert to disk cache");

    // There should not be anything for the key in memory cache
    assert!(!layer_cache.memory_cache().has_key(&skid_row));

    // Insert through the layer cache
    layer_cache
        .insert("skid row", "slave to the grind".into())
        .await
        .expect("cannot insert into the cache");

    // There should be an entry in memory now
    assert!(layer_cache.memory_cache.has_key(&skid_row));
}

#[tokio::test]
async fn in_memory_but_not_on_disk_insert() {
    let layer_cache = make_layer_cache();

    let skid_row = "skid row";

    // Insert the object directly to memory cache
    layer_cache
        .memory_cache
        .insert_value("skid row", "slave to the grind".into())
        .await;

    // There should not be anything for the key in disk cache
    assert!(!layer_cache
        .disk_cache
        .contains_key(&skid_row)
        .expect("cannot check if key exists in disk cache"));

    // Insert through the layer cache
    layer_cache
        .insert("skid row", "slave to the grind".into())
        .await
        .expect("cannot insert into the cache");

    // There should be an entry in disk now
    assert!(layer_cache
        .disk_cache
        .contains_key(&skid_row)
        .expect("cannot read from disk cache"));
}

#[tokio::test]
async fn get_inserts_to_memory() {
    let layer_cache = make_layer_cache();

    let skid_row = "skid row";

    let postcard_serialized = postcard::to_stdvec("slave to the grind").expect("should serialize");

    layer_cache
        .disk_cache
        .insert("skid row", &postcard_serialized)
        .expect("failed to insert to disk cache");
    assert!(!layer_cache.memory_cache.has_key(&skid_row));

    layer_cache
        .get(&skid_row)
        .await
        .expect("error getting object from cache")
        .expect("object not in cachche");

    assert!(layer_cache.memory_cache.has_key(&skid_row));
}

#[tokio::test]
async fn multiple_mokas_single_sled() {
    let count = 10_000;
    let tempdir = tempfile::tempdir().expect("cannot create tempdir");
    let db = sled::open(tempdir).expect("unable to open sled database");
    let cache_even: Cache<[u8; 8], String> = Cache::new(count);

    let even_tree_name = "even_numbers";
    let odd_tree_name = "odd_numbers";

    let layer_cache_even = LayerCache::new(db.clone(), even_tree_name, Box::new(cache_even))
        .expect("cannot create layer cache");

    let cache_odd: Cache<[u8; 8], String> = Cache::new(count);
    let layer_cache_odd = LayerCache::new(db.clone(), odd_tree_name, Box::new(cache_odd))
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

    task_set.spawn(async move {
        for i in 0..(count * 2) {
            if i % 2 == 0 {
                let (key, value) = make_u64_kv(i);
                layer_cache_even
                    .insert(key, value)
                    .await
                    .expect("unable to insert");
            }
        }
    });

    task_set.spawn(async move {
        for i in 0..(count * 2) {
            if i % 2 != 0 {
                let (key, value) = make_u64_kv(i);
                layer_cache_odd
                    .insert(key, value)
                    .await
                    .expect("unable to insert");
            }
        }
    });

    while let Some(_) = task_set.join_next().await {}

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
            .map(|value| postcard::from_bytes(value.as_ref()).ok())
            .flatten();

        let odd_tree_value: Option<String> = odd_tree
            .get(key)
            .expect("able to get odd key value")
            .map(|value| postcard::from_bytes(value.as_ref()).ok())
            .flatten();

        if i % 2 == 0 {
            assert_eq!(Some(value), even_tree_value);
            assert_eq!(None, odd_tree_value);
        } else {
            assert_eq!(Some(value), odd_tree_value);
            assert_eq!(None, even_tree_value);
        }
    }
}
