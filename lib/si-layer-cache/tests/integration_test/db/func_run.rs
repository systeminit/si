use chrono::Utc;
use si_layer_cache::memory_cache::MemoryCacheConfig;
use std::collections::HashSet;
use std::{sync::Arc, time::Duration};

use si_events::{
    Actor, ChangeSetId, ContentHash, FuncBackendKind, FuncBackendResponseType, FuncKind, FuncRun,
    FuncRunBuilder, FuncRunId, Tenancy, UserPk, WorkspacePk,
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
    let dbfile = disk_cache_path(&tempdir, "jesawyer");
    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("func_run_write_to_db").await,
        setup_nats_client(Some("func_run_write_to_db".to_string())).await,
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

    let func_run = create_func_run(actor, tenancy, "joshua graham");
    let key_str: Arc<str> = func_run.id().to_string().into();
    let value: Arc<FuncRun> = Arc::new(func_run);

    ldb.func_run()
        .write(value.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run().cache.memory_cache().get(&key_str).await;
    assert_eq!(value.id(), in_memory.expect("func run not in memory").id());

    // Are we on disk?
    let on_disk_postcard = ldb
        .func_run()
        .cache
        .disk_cache()
        .get(key_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: FuncRun =
        serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), on_disk.id());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRun =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), in_pg.id());
}

#[tokio::test]
async fn update() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let dbfile = disk_cache_path(&tempdir, "jesawyer");
    let db = setup_pg_db("func_run_update_to_db").await;
    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        db.clone(),
        setup_nats_client(Some("func_run_update_to_db".to_string())).await,
        MemoryCacheConfig::default(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let dbfile = disk_cache_path(&tempdir, "rainbow");
    let (ldb_remote, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        db,
        setup_nats_client(Some("func_run_update_to_db".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );

    let func_run = create_func_run(actor, tenancy, "lindsey buckingham");
    let key_str: Arc<str> = func_run.id().to_string().into();
    let value: Arc<FuncRun> = Arc::new(func_run);

    ldb.func_run()
        .write(value.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run().cache.memory_cache().get(&key_str).await;
    assert_eq!(value.id(), in_memory.expect("func run not in memory").id());

    // Are we on disk?
    let on_disk_postcard = ldb
        .func_run()
        .cache
        .disk_cache()
        .get(key_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: FuncRun =
        serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), on_disk.id());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRun =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), in_pg.id());
    assert_eq!(value.state(), in_pg.state());

    // Update the state to success
    let mut update_func_run_inner = Arc::unwrap_or_clone(value);
    update_func_run_inner.set_state_to_success();
    let update_func_run = Arc::new(update_func_run_inner);

    ldb.func_run()
        .write(update_func_run.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run().cache.memory_cache().get(&key_str).await;
    assert_eq!(
        update_func_run.state(),
        in_memory.expect("func run not in memory").state(),
        "updated in memory state"
    );

    // Are we on disk?
    let on_disk_postcard = ldb
        .func_run()
        .cache
        .disk_cache()
        .get(key_str.clone())
        .await
        .expect("cannot get from disk cache");
    let on_disk: FuncRun =
        serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
    assert_eq!(update_func_run.state(), on_disk.state());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRun =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(update_func_run.state(), in_pg.state());

    let max_check_count = 10;
    let mut memory_check_count = 0;
    while memory_check_count <= max_check_count {
        let in_memory = ldb_remote
            .func_run()
            .cache
            .memory_cache()
            .get(&key_str)
            .await;
        match in_memory {
            Some(value) => {
                assert_eq!(update_func_run.state(), value.state());
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
            .func_run()
            .cache
            .disk_cache()
            .get(key_str.clone())
            .await
        {
            Ok(on_disk_postcard) => {
                let on_disk: FuncRun =
                    serialize::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
                assert_eq!(update_func_run.state(), on_disk.state());
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
async fn write_and_read_many_for_workspace_id() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "fnv");

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("func_run_write_and_read_many_for_workspace_id").await,
        setup_nats_client(Some(
            "fun_run_write_and_read_many_for_workspace_id".to_string(),
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

    let values = vec![
        Arc::new(create_func_run(actor, tenancy, "dead money")),
        Arc::new(create_func_run(actor, tenancy, "honest hearts")),
        Arc::new(create_func_run(actor, tenancy, "old world blues")),
        Arc::new(create_func_run(actor, tenancy, "lonesome road")),
    ];
    let expected: HashSet<FuncRunId> = HashSet::from_iter(values.iter().map(|v| v.id()));

    for value in values {
        ldb.func_run()
            .write(value.clone(), None, tenancy, actor)
            .await
            .expect("failed to write to layerdb");
    }

    let read_many_in_workspace_values = ldb
        .func_run()
        .read_many_for_workspace(tenancy.workspace_pk)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");

    assert_eq!(
        expected,
        HashSet::from_iter(read_many_in_workspace_values.iter().map(|v| v.id()))
    );
}

fn create_func_run(actor: Actor, tenancy: Tenancy, function_name: impl Into<String>) -> FuncRun {
    let func_run_create_time = Utc::now();
    FuncRunBuilder::default()
        .actor(actor)
        .tenancy(tenancy)
        .component_id(None)
        .attribute_value_id(None)
        .backend_kind(FuncBackendKind::JsAction)
        .backend_response_type(FuncBackendResponseType::Action)
        .function_name(function_name.into())
        .function_kind(FuncKind::Action)
        .function_args_cas_address(ContentHash::default())
        .function_code_cas_address(ContentHash::default())
        .created_at(func_run_create_time)
        .updated_at(func_run_create_time)
        .build()
        .expect("could not build func run")
}
