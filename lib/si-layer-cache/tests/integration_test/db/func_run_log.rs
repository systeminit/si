use si_layer_cache::memory_cache::MemoryCacheConfig;
use std::{sync::Arc, time::Duration};

use si_events::{
    Actor, ChangeSetId, FuncRunId, FuncRunLog, OutputLine, Tenancy, UserPk, WorkspacePk,
};
use si_layer_cache::db::serialize;
use si_layer_cache::LayerDb;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::integration_test::{disk_cache_path, setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<String, String, String>;

#[tokio::test]
async fn write_to_db() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let dbfile = disk_cache_path(&tempdir, "esp");
    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("func_run_log_write_to_db").await,
        setup_nats_client(Some("func_run_log_write_to_db".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );
    let func_run_log = FuncRunLog::new(FuncRunId::new(), tenancy);
    let key_str: Arc<str> = func_run_log.id().to_string().into();
    let value: Arc<FuncRunLog> = Arc::new(func_run_log);

    ldb.func_run_log()
        .write(value.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run_log().cache.memory_cache().get(&key_str).await;
    assert_eq!(
        value.id(),
        in_memory.expect("func run log not in memory").id()
    );

    // Are we on disk?
    let on_disk_postcard = ldb
        .func_run_log()
        .cache
        .disk_cache()
        .get(key_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: FuncRunLog =
        serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), on_disk.id());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run_log()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRunLog =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), in_pg.id());
}

#[tokio::test]
async fn update() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let dbfile = disk_cache_path(&tempdir, "sevenmarythree");
    let db = setup_pg_db("func_run_log_update_to_db").await;
    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        db.clone(),
        setup_nats_client(Some("func_run_log_update_to_db".to_string())).await,
        MemoryCacheConfig::default(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let dbfile = disk_cache_path(&tempdir, "tea & coffee");
    let (ldb_remote, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        db,
        setup_nats_client(Some("func_run_log_update_to_db".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );
    let func_run_log = FuncRunLog::new(FuncRunId::new(), tenancy);
    let key_str: Arc<str> = func_run_log.id().to_string().into();
    let value: Arc<FuncRunLog> = Arc::new(func_run_log);

    ldb.func_run_log()
        .write(value.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run_log().cache.memory_cache().get(&key_str).await;
    assert_eq!(
        value.id(),
        in_memory.expect("func run log not in memory").id()
    );

    // Are we on disk?
    let on_disk_postcard = ldb
        .func_run_log()
        .cache
        .disk_cache()
        .get(key_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: FuncRunLog =
        serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), on_disk.id());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run_log()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRunLog =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), in_pg.id());
    assert_eq!(value.logs(), in_pg.logs());

    // Update the logs
    let mut update_func_run_log_inner = Arc::unwrap_or_clone(value);
    let dummy_text = "DUMMY".to_string();
    update_func_run_log_inner.push_log(OutputLine {
        stream: dummy_text.clone(),
        execution_id: dummy_text.clone(),
        level: dummy_text.clone(),
        group: None,
        message: dummy_text,
        timestamp: 0,
    });
    let update_func_run_log = Arc::new(update_func_run_log_inner);

    ldb.func_run_log()
        .write(update_func_run_log.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run_log().cache.memory_cache().get(&key_str).await;
    assert_eq!(
        update_func_run_log.logs(),
        in_memory.expect("func run log not in memory").logs(),
        "updated in memory logs"
    );

    // Are we on disk?
    let on_disk_postcard = ldb
        .func_run_log()
        .cache
        .disk_cache()
        .get(key_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: FuncRunLog =
        serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(update_func_run_log.logs(), on_disk.logs());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run_log()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRunLog =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(update_func_run_log.logs(), in_pg.logs());

    let max_check_count = 10;
    let mut memory_check_count = 0;
    while memory_check_count <= max_check_count {
        let in_memory = ldb_remote
            .func_run_log()
            .cache
            .memory_cache()
            .get(&key_str)
            .await;
        match in_memory {
            Some(value) => {
                assert_eq!(update_func_run_log.logs(), value.logs());
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

    let mut disk_check_count = 0;
    while disk_check_count <= max_check_count {
        match ldb_remote
            .func_run_log()
            .cache
            .disk_cache()
            .get(key_str.clone())
            .await
        {
            Ok(on_disk_postcard) => {
                let on_disk: FuncRunLog =
                    serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
                assert_eq!(update_func_run_log.logs(), on_disk.logs());
                break;
            }
            Err(_e) => {
                disk_check_count += 1;
                tokio::time::sleep_until(Instant::now() + Duration::from_millis(1)).await;
            }
        }
    }
    assert_ne!(
        max_check_count, disk_check_count,
        "value did not arrive in the remote disk cache within 10ms"
    );
}

#[tokio::test]
async fn write_and_get_for_func_run_id() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "cumbersome");

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("func_run_log_write_and_read_many_for_func_run_id").await,
        setup_nats_client(Some(
            "func_run_log_write_and_read_many_for_func_run_id".to_string(),
        ))
        .await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );

    let func_run_id = FuncRunId::new();
    let func_run_log = FuncRunLog::new(func_run_id, tenancy);
    let value: Arc<FuncRunLog> = Arc::new(func_run_log);

    ldb.func_run_log()
        .write(value.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    let read_value = ldb
        .func_run_log()
        .get_for_func_run_id(func_run_id)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");

    assert_eq!(value.id(), read_value.id());
}
