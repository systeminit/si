use std::{
    sync::Arc,
    time::Duration,
};

use si_events::{
    Actor,
    ChangeSetId,
    Tenancy,
    UserPk,
    WorkspacePk,
};
use si_layer_cache::{
    LayerDb,
    db::serialize,
    persister::PersistStatus,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::integration_test::{
    make_test_layerdb_config,
    setup_compute_executor,
    setup_nats_client,
    setup_pg_db,
};

type TestLayerDb = LayerDb<String, String, String, String, String, String, String>;

#[tokio::test]
async fn write_to_db() {
    let token = CancellationToken::new();

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        setup_pg_db("workspace_snapshot_write_to_db").await,
        setup_nats_client(Some("workspace_snapshot_write_to_db".to_string())).await,
        setup_compute_executor(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let value: Arc<String> = Arc::new("pantera".into());
    let (key, status) = ldb
        .workspace_snapshot()
        .write(
            value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::User(UserPk::new()),
        )
        .expect("failed to write to layerdb");

    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Write failed; {e}"),
    }

    let key_str: Arc<str> = key.to_string().into();

    // Are we in memory?
    let in_memory = ldb
        .workspace_snapshot()
        .cache
        .cache()
        .get(key_str.clone())
        .await;
    assert_eq!(Some(value.clone()), in_memory);

    // Are we in pg?
    let in_pg_postcard = ldb
        .workspace_snapshot()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: String =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.as_ref(), &in_pg);
}

#[tokio::test]
async fn evict_from_db() {
    let token = CancellationToken::new();

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        setup_pg_db("workspace_snapshot_evict_from_db").await,
        setup_nats_client(Some("workspace_snapshot_evict_from_db".to_string())).await,
        setup_compute_executor(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let value: Arc<String> = Arc::new("pantera".into());
    let (key, status) = ldb
        .workspace_snapshot()
        .write(
            value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::User(UserPk::new()),
        )
        .expect("failed to write to layerdb");

    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Write failed; {e}"),
    }

    let key_str: Arc<str> = key.to_string().into();

    let status = ldb
        .workspace_snapshot()
        .evict(
            &key,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::System,
        )
        .expect("cannot evict local data");
    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Eviction failed; {e}"),
    }

    // wait for the changes to happen in CI, which can be very slow
    // like, slower than you think even
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Are we in memory?
    let in_memory = ldb
        .workspace_snapshot()
        .cache
        .cache()
        .get(key_str.clone())
        .await;
    assert!(in_memory.is_none());

    assert!(
        ldb.workspace_snapshot()
            .cache
            .pg()
            .get(&key_str)
            .await
            .expect("error getting data from pg")
            .is_none(),
        "found item in database when it should have been evicted"
    );
}

#[tokio::test]
async fn evictions_are_gossiped() {
    let token = CancellationToken::new();

    let db = setup_pg_db("workspace_snapshot_evictions_are_gossiped").await;

    // First, we need a layerdb for slash
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        db.clone(),
        setup_nats_client(Some(
            "workspace_snapshot_evictions_are_gossiped".to_string(),
        ))
        .await,
        setup_compute_executor(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // Then, we need a layerdb for axl
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        db,
        setup_nats_client(Some(
            "workspace_snapshot_evictions_are_gossiped".to_string(),
        ))
        .await,
        setup_compute_executor(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    let value: Arc<String> = Arc::new("pantera".into());
    let (key, status) = ldb_slash
        .workspace_snapshot()
        .write(
            value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::User(UserPk::new()),
        )
        .expect("failed to write to layerdb");
    assert!(
        matches!(
            status.get_status().await.expect("failed to get status"),
            PersistStatus::Finished
        ),
        "persister failed"
    );

    let pk_str: Arc<str> = key.to_string().into();

    let max_check_count = 100;

    let mut memory_check_count = 0;
    while memory_check_count <= max_check_count {
        let in_memory = ldb_axl
            .workspace_snapshot()
            .cache
            .cache()
            .get(pk_str.clone())
            .await;
        match in_memory {
            Some(value) => {
                assert_eq!(value.clone(), value);
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

    // Are we in pg?
    let in_pg_postcard = ldb_axl
        .workspace_snapshot()
        .cache
        .pg()
        .get(&pk_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: String =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.as_ref(), &in_pg);

    // Evict!
    let status = ldb_slash
        .workspace_snapshot()
        .evict(
            &key,
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
            Actor::System,
        )
        .expect("cannot evict local data");
    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Eviction failed; {e}"),
    }

    let max_check_count = 100;

    let mut memory_check_count = 0;
    while memory_check_count < max_check_count {
        let in_memory = ldb_axl
            .workspace_snapshot()
            .cache
            .cache()
            .get(pk_str.clone())
            .await;
        match in_memory {
            Some(_value) => {
                memory_check_count += 1;
                tokio::time::sleep_until(Instant::now() + Duration::from_millis(1)).await;
            }
            None => {
                break;
            }
        }
    }
    assert_ne!(
        max_check_count, memory_check_count,
        "value did not evict from the remote memory cache within 100ms"
    );
}
