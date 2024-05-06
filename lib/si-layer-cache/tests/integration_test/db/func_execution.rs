use si_events::ulid::Ulid;
use si_events::{
    CasValue, FuncExecution, FuncExecutionKey, FuncExecutionMessage, FuncExecutionState,
};
use si_layer_cache::memory_cache::MemoryCacheConfig;
use si_layer_cache::LayerDb;
use tokio_util::sync::CancellationToken;

use crate::integration_test::{disk_cache_path, setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<CasValue, String, String>;

#[tokio::test]
async fn write() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let dbfile = disk_cache_path(&tempdir, "mbd");
    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("fe_write").await,
        setup_nats_client(Some("fe_write".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let mut key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new(), Ulid::new());
    let value = FuncExecution::new("funky".to_string(), FuncExecutionState::Create);
    key = ldb
        .func_execution()
        .write(key.clone(), value.clone().into())
        .await
        .expect("failed to write to layerdb");
    assert!(key.func_execution_id().is_some());

    // Are we in pg?
    let in_pg = ldb
        .func_execution()
        .read(key)
        .await
        .expect("error getting data from pg")
        .expect("no fe object in pg");
    assert_eq!(value, in_pg);
}

#[tokio::test]
async fn write_with_message() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let dbfile = disk_cache_path(&tempdir, "mbd");
    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("fe_write_with_maessage").await,
        setup_nats_client(Some("fe_write_with_message".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let mut key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new(), Ulid::new());
    let value = FuncExecution::new("funky".to_string(), FuncExecutionState::Create);
    key = ldb
        .func_execution()
        .write(key.clone(), value.clone().into())
        .await
        .expect("failed to write to layerdb");
    assert!(key.func_execution_id().is_some());

    // Are we in pg?
    let in_pg = ldb
        .func_execution()
        .read(key.clone())
        .await
        .expect("error getting data from pg")
        .expect("no fe object in pg");
    assert_eq!(in_pg, value);

    let message = FuncExecutionMessage::new("kungfu kenny".to_string());

    // add a log message for this func
    key = ldb
        .func_execution()
        .write_message(key.clone(), message.clone())
        .await
        .expect("should insert message");

    // make sure we updated the key to include the message_id
    assert!(key.message_id().is_some());

    let message_in_pg = ldb
        .func_execution()
        .read_message(key)
        .await
        .expect("error getting data from pg")
        .expect("no fe object in pg");

    assert_eq!(message, message_in_pg);
}

#[tokio::test]
async fn write_and_read_many() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "mbd");

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("fe_write_and_read_many").await,
        setup_nats_client(Some("fe_write_and_read_many".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let values = vec![
        FuncExecution::new("spring break 1989".to_string(), FuncExecutionState::Create),
        FuncExecution::new("comin home".to_string(), FuncExecutionState::Create),
        FuncExecution::new("lost river".to_string(), FuncExecutionState::Create),
        FuncExecution::new("foxglove".to_string(), FuncExecutionState::Create),
    ];
    let mut keys = vec![];

    for value in &values {
        let mut key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new(), Ulid::new());
        key = ldb
            .func_execution()
            .write(key.clone(), value.clone().into())
            .await
            .expect("failed to write to layerdb");

        key = ldb
            .func_execution()
            .write_message(
                key.clone(),
                FuncExecutionMessage::new(value.name().to_string()),
            )
            .await
            .expect("should insert message");

        keys.push(key);
    }

    let read_values = ldb
        .func_execution()
        .read_many(&keys)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");

    assert_eq!(&read_values.len(), &values.len());
    for value in read_values {
        assert!(values.contains(&value));
    }
}

#[tokio::test]
async fn read_by_component_id() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "mbd");

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("fe_by_component_id").await,
        setup_nats_client(Some("fe_by_component_id".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let values = vec![
        FuncExecution::new("spring break 1989".to_string(), FuncExecutionState::Create),
        FuncExecution::new("comin home".to_string(), FuncExecutionState::Create),
        FuncExecution::new("lost river".to_string(), FuncExecutionState::Create),
        FuncExecution::new("foxglove".to_string(), FuncExecutionState::Create),
    ];
    let mut keys = vec![];
    let component_id = Ulid::new();

    for value in &values {
        let mut key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), component_id, Ulid::new());
        key = ldb
            .func_execution()
            .write(key.clone(), value.clone().into())
            .await
            .expect("failed to write to layerdb");

        key = ldb
            .func_execution()
            .write_message(
                key.clone(),
                FuncExecutionMessage::new(value.name().to_string()),
            )
            .await
            .expect("should insert message");

        keys.push(key);
    }

    // write another one for a different component just to make sure it doesn't show up
    let key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new(), Ulid::new());
    let value = FuncExecution::new("no oath, no spell".to_string(), FuncExecutionState::Create);
    ldb.func_execution()
        .write(key.clone(), value.clone().into())
        .await
        .expect("failed to write to layerdb");

    // go get em
    let read_values = ldb
        .func_execution()
        .read_many_by_component_id(&component_id)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");

    assert_eq!(&read_values.len(), &values.len());
    for value in read_values {
        assert!(values.contains(&value));
    }
}

#[tokio::test]
async fn read_by_prototype_id() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "mbd");

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("fe_by_prototype_id").await,
        setup_nats_client(Some("fe_by_prototype_id".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let values = vec![
        FuncExecution::new("spring break 1989".to_string(), FuncExecutionState::Create),
        FuncExecution::new("comin home".to_string(), FuncExecutionState::Create),
        FuncExecution::new("lost river".to_string(), FuncExecutionState::Create),
        FuncExecution::new("foxglove".to_string(), FuncExecutionState::Create),
    ];
    let mut keys = vec![];
    let prototype_id = Ulid::new();

    for value in &values {
        let mut key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new(), prototype_id);
        key = ldb
            .func_execution()
            .write(key.clone(), value.clone().into())
            .await
            .expect("failed to write to layerdb");

        key = ldb
            .func_execution()
            .write_message(
                key.clone(),
                FuncExecutionMessage::new(value.name().to_string()),
            )
            .await
            .expect("should insert message");

        keys.push(key);
    }

    // write another one for a different prototype just to make sure it doesn't show up
    let key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new(), Ulid::new());
    let value = FuncExecution::new("no oath, no spell".to_string(), FuncExecutionState::Create);
    ldb.func_execution()
        .write(key.clone(), value.clone().into())
        .await
        .expect("failed to write to layerdb");
    let read_values = ldb
        .func_execution()
        .read_many_by_prototype_id(&prototype_id)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");

    assert_eq!(&read_values.len(), &values.len());
    for value in read_values {
        assert!(values.contains(&value));
    }
}

#[tokio::test]
async fn read_by_component_id_and_prototype_id() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "mbd");

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        dbfile,
        setup_pg_db("fe_by_component_id_and_prototype_id").await,
        setup_nats_client(Some("fe_by_component_id_and_prototype_id".to_string())).await,
        MemoryCacheConfig::default(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let values = vec![
        FuncExecution::new("spring break 1989".to_string(), FuncExecutionState::Create),
        FuncExecution::new("comin home".to_string(), FuncExecutionState::Create),
        FuncExecution::new("lost river".to_string(), FuncExecutionState::Create),
        FuncExecution::new("foxglove".to_string(), FuncExecutionState::Create),
    ];

    let mut keys = vec![];
    let component_id = Ulid::new();
    let prototype_id = Ulid::new();

    for value in &values {
        let mut key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), component_id, prototype_id);
        key = ldb
            .func_execution()
            .write(key.clone(), value.clone().into())
            .await
            .expect("failed to write to layerdb");

        key = ldb
            .func_execution()
            .write_message(
                key.clone(),
                FuncExecutionMessage::new(value.name().to_string()),
            )
            .await
            .expect("should insert message");

        keys.push(key);
    }

    // write another one for a different component just to make sure it doesn't show up
    let key = FuncExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new(), prototype_id);
    let value = FuncExecution::new("no oath, no spell".to_string(), FuncExecutionState::Create);
    ldb.func_execution()
        .write(key.clone(), value.clone().into())
        .await
        .expect("failed to write to layerdb");

    let read_values = ldb
        .func_execution()
        .read_many_by_component_id_and_prototype_id(&component_id, &prototype_id)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");

    assert_eq!(&read_values.len(), &values.len());
    for value in read_values {
        assert!(values.contains(&value));
    }
}
