use std::sync::Arc;

use si_events::{CasValue, FunctionExecutionKey};
use si_layer_cache::LayerDb;
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::integration_test::{disk_cache_path, setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<CasValue, String, String, String>;

#[tokio::test]
async fn write_to_db() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let dbfile = disk_cache_path(&tempdir, "mbd");
    let (ldb, _): (TestLayerDb, _) = LayerDb::initialize(
        dbfile,
        setup_pg_db("fe_write_to_db").await,
        setup_nats_client(Some("fe_write_to_db".to_string())).await,
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let key = FunctionExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new());
    let value: Arc<String> = Arc::new("go to the light".to_string());
    ldb.function_execution()
        .write(key.clone(), value.clone())
        .await
        .expect("failed to write to layerdb");

    // Are we in pg?
    let in_pg = ldb
        .function_execution()
        .read(&key)
        .await
        .expect("error getting data from pg")
        .expect("no fe object in pg");
    assert_eq!(value.as_ref(), in_pg.as_ref());
}

#[tokio::test]
async fn write_and_read_many() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");

    let dbfile = disk_cache_path(&tempdir, "mbd");

    let (ldb, _): (TestLayerDb, _) = LayerDb::initialize(
        dbfile,
        setup_pg_db("fe_write_and_read_many").await,
        setup_nats_client(Some("fe_write_and_read_many".to_string())).await,
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let values: Vec<Arc<String>> = vec![
        Arc::new("spring break 1989".to_string()),
        Arc::new("comin home".to_string()),
        Arc::new("lost river".to_string()),
        Arc::new("foxglove".to_string()),
    ];

    let mut keys = vec![];

    for value in &values {
        let key = FunctionExecutionKey::new(Ulid::new(), Ulid::new(), Ulid::new());
        keys.push(Arc::new(key.clone()));
        ldb.function_execution()
            .write(key.clone(), value.clone())
            .await
            .expect("failed to write to layerdb");
    }

    let read_values = ldb
        .function_execution()
        .read_many(keys.as_slice())
        .await
        .expect("should be able to read");

    assert_eq!(&read_values.len(), &values.len());
    for value in read_values.values().collect::<Vec<_>>() {
        assert!(values.contains(value));
    }
}
