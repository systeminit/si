use std::{sync::Arc, time::Duration};

use si_events::{Actor, CasValue, ChangeSetId, ContentHash, Tenancy, UserPk, WorkspacePk};
use si_layer_cache::{persister::PersistStatus, LayerDb};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::integration_test::{disk_cache_path, setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<CasValue, String, String, String>;

#[tokio::test]
async fn write_to_db() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let dbfile = disk_cache_path(&tempdir, "slash");
    let (ldb, _): (TestLayerDb, _) = LayerDb::initialize(
        dbfile,
        setup_pg_db("cas_write_to_db").await,
        setup_nats_client(Some("cas_write_to_db".to_string())).await,
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let cas_value: Arc<CasValue> = Arc::new(serde_json::json!("stone sour").into());
    let (cas_pk, status) = ldb
        .cas()
        .write(
            cas_value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::User(UserPk::new()),
        )
        .await
        .expect("failed to write to layerdb");

    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Write failed; {e}"),
    }

    let cas_pk_str: Arc<str> = cas_pk.to_string().into();

    // Are we in memory?
    let in_memory = ldb.cas().cache.memory_cache().get(&cas_pk_str).await;
    assert_eq!(Some(cas_value.clone()), in_memory);

    // Are we on disk?
    let on_disk_postcard = ldb
        .cas()
        .cache
        .disk_cache()
        .get(cas_pk_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: CasValue =
        postcard::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(cas_value.as_ref(), &on_disk);

    // Are we in pg?
    let in_pg_postcard = ldb
        .cas()
        .cache
        .pg()
        .get(&cas_pk_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: CasValue =
        postcard::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(cas_value.as_ref(), &in_pg);
}

#[tokio::test]
async fn write_and_read_many() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "slash");

    let (ldb, _): (TestLayerDb, _) = LayerDb::initialize(
        dbfile,
        setup_pg_db("cas_write_and_read_many").await,
        setup_nats_client(Some("cas_write_and_read_many".to_string())).await,
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let cas_values: Vec<Arc<CasValue>> = vec![
        Arc::new(serde_json::json!("stone sour").into()),
        Arc::new(serde_json::json!("tone flour").into()),
        Arc::new(serde_json::json!("bologna chowder").into()),
        Arc::new(serde_json::json!("waaagh").into()),
    ];
    let mut keys: Vec<ContentHash> = vec![];

    for cas_value in &cas_values {
        let (cas_pk, status) = ldb
            .cas()
            .write(
                cas_value.clone(),
                None,
                Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
                Actor::User(UserPk::new()),
            )
            .await
            .expect("failed to write to layerdb");
        keys.push(cas_pk);
        match status.get_status().await.expect("failed to get status") {
            PersistStatus::Finished => {}
            PersistStatus::Error(e) => panic!("Write failed; {e}"),
        }
    }

    let read_values = ldb
        .cas()
        .read_many(&keys)
        .await
        .expect("should be able to read");

    for value in read_values.values().collect::<Vec<_>>() {
        assert!(cas_values.contains(value));
    }
}

#[tokio::test]
async fn cold_read_from_db() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "slash");

    let (ldb, _): (TestLayerDb, _) = LayerDb::initialize(
        dbfile,
        setup_pg_db("cas_cold_read_from_db").await,
        setup_nats_client(Some("cas_cold_read_from_db".to_string())).await,
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layerdb");

    let cas_value: Arc<CasValue> = Arc::new(serde_json::json!("stone sour").into());
    let (cas_pk, status) = ldb
        .cas()
        .write(
            cas_value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::User(UserPk::new()),
        )
        .await
        .expect("failed to write to layerdb");
    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Write failed; {e}"),
    }

    let cas_pk_str: Arc<str> = cas_pk.to_string().into();

    // Delete from memory and disk
    ldb.cas().cache.memory_cache().remove(&cas_pk_str).await;
    let not_in_memory = ldb.cas().cache.memory_cache().get(&cas_pk_str).await;
    assert_eq!(not_in_memory, None);

    ldb.cas()
        .cache
        .disk_cache()
        .remove(cas_pk_str.clone())
        .await
        .expect("cannot remove from disk");
    let not_on_disk = ldb.cas().cache.disk_cache().get(cas_pk_str.clone()).await;
    assert!(not_on_disk.is_err());

    // Read the data from the cache
    let data = ldb
        .cas()
        .read(&cas_pk)
        .await
        .expect("cannot read from layerdb")
        .expect("data not in layerdb");

    assert_eq!(&cas_value, &data);

    // Are we in memory?
    let in_memory = ldb.cas().cache.memory_cache().get(&cas_pk_str).await;
    assert_eq!(Some(cas_value.clone()), in_memory);

    // Are we on disk?
    let on_disk_postcard = ldb
        .cas()
        .cache
        .disk_cache()
        .get(cas_pk_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: CasValue =
        postcard::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(cas_value.as_ref(), &on_disk);

    // Are we in pg?
    let in_pg_postcard = ldb
        .cas()
        .cache
        .pg()
        .get(&cas_pk_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: CasValue =
        postcard::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(cas_value.as_ref(), &in_pg);
}

#[tokio::test]
async fn writes_are_gossiped() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");

    let db = setup_pg_db("cas_writes_are_gossiped").await;

    // First, we need a layerdb for slash
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("cas_writes_are_gossiped".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // Then, we need a layerdb for axl
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db,
        setup_nats_client(Some("cas_write_to_db".to_string())).await,
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    let big_string = "a".repeat(1_000_000);
    let cas_value: Arc<CasValue> = Arc::new(CasValue::String(big_string));
    let (cas_pk, status) = ldb_slash
        .cas()
        .write(
            cas_value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::User(UserPk::new()),
        )
        .await
        .expect("failed to write to layerdb");
    assert!(
        matches!(
            status.get_status().await.expect("failed to get status"),
            PersistStatus::Finished
        ),
        "persister failed"
    );

    let cas_pk_str: Arc<str> = cas_pk.to_string().into();

    let max_check_count = 10;

    let mut memory_check_count = 0;
    while memory_check_count <= max_check_count {
        let in_memory = ldb_axl.cas().cache.memory_cache().get(&cas_pk_str).await;
        match in_memory {
            Some(value) => {
                assert_eq!(cas_value.clone(), value);
                break;
            }
            None => {
                memory_check_count += 1;
                tokio::time::sleep_until(Instant::now() + Duration::from_millis(1)).await;
            }
        }
    }
    assert_ne!(
        max_check_count, memory_check_count,
        "value did not arrive in the remote memory cache within 10ms"
    );

    // Are we on disk?
    let mut disk_check_count = 0;
    while disk_check_count <= max_check_count {
        match ldb_axl
            .cas()
            .cache
            .disk_cache()
            .get(cas_pk_str.clone())
            .await
        {
            Ok(on_disk_postcard) => {
                let on_disk: CasValue =
                    postcard::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
                assert_eq!(cas_value.as_ref(), &on_disk);
                break;
            }
            Err(_e) => {
                disk_check_count += 1;
                tokio::time::sleep_until(Instant::now() + Duration::from_millis(1)).await;
            }
        }
    }
    assert_ne!(
        max_check_count, memory_check_count,
        "value did not arrive in the remote disk cache within 10ms"
    );

    // Are we in pg?
    let in_pg_postcard = ldb_axl
        .cas()
        .cache
        .pg()
        .get(&cas_pk_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: CasValue =
        postcard::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(cas_value.as_ref(), &in_pg);
}

#[tokio::test(flavor = "multi_thread")]
async fn stress_test() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");

    let db = setup_pg_db("stress_test").await;

    // First, we need a layerdb for slash
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("stress_test".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    dbg!(ldb_slash.instance_id().to_string());

    // Then, we need a layerdb for axl
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db,
        setup_nats_client(Some("stress_test".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");
    dbg!(ldb_axl.instance_id().to_string());

    let values = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    ];
    let mut write_join_set: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();
    let mut read_join_set: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();

    for v in values {
        let big_string: Arc<String> = Arc::new(v.repeat(10_000_000));
        let cas_value = Arc::new(CasValue::String(big_string.to_string()));
        let postcard_value =
            postcard::to_stdvec(&cas_value).expect("cannot deserialize big ass string");
        let cas_pk_string = ContentHash::new(&postcard_value).to_string();
        let ldb_slash_task = ldb_slash.clone();
        let _write_big_string = big_string.clone();
        let write_cas_value = cas_value.clone();
        write_join_set.spawn(async move {
            let (_cas_pk, _status) = ldb_slash_task
                .cas()
                .write(
                    write_cas_value,
                    None,
                    Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
                    Actor::User(UserPk::new()),
                )
                .await
                .expect("failed to write to layerdb");
        });

        let ldb_axl_task = ldb_axl.clone();
        read_join_set.spawn(async move {
            let max_check_count = 3000;
            let mut memory_check_count = 0;
            while memory_check_count < max_check_count {
                let in_memory = ldb_axl_task
                    .cas()
                    .cache
                    .memory_cache()
                    .get(&cas_pk_string)
                    .await;
                match in_memory {
                    Some(value) => {
                        let cas_value: Arc<CasValue> =
                            Arc::new(CasValue::String(big_string.to_string()));
                        assert_eq!(cas_value, value);
                        break;
                    }
                    None => {
                        memory_check_count += 1;
                        tokio::time::sleep_until(Instant::now() + Duration::from_millis(1)).await;
                    }
                }
            }
            assert_ne!(
                max_check_count, memory_check_count,
                "value did not arrive in the remote memory cache within 3 seconds"
            );
        });
    }

    let time = tokio::time::Instant::now();
    while let Some(res) = write_join_set.join_next().await {
        if let Err(e) = res {
            panic!("Write failed {}", e);
        }
    }
    println!("writes are all sent: {:?}", time.elapsed());

    let time = tokio::time::Instant::now();
    while let Some(res) = read_join_set.join_next().await {
        if let Err(e) = res {
            println!("read failed: {:?}", time.elapsed());
            panic!("Read failed {}", e);
        }
        println!("read succeeded: {:?}", time.elapsed());
    }
    println!("reads are all read: {:?}", time.elapsed());
    token.cancel();
}
