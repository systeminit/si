use std::{sync::Arc, time::Duration};

use si_events::{Actor, CasValue, ChangeSetId, ContentHash, Tenancy, UserPk, WorkspacePk};
use si_layer_cache::{persister::PersistStatus, LayerDb};
use tokio::time::Instant;

use crate::integration_test::{setup_nats_client, setup_pg_db};

#[tokio::test]
async fn write_to_db() {
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let ldb = LayerDb::new(
        tempdir,
        setup_pg_db("cas_write_to_db").await,
        setup_nats_client(Some("cas_write_to_db".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");

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

    let cas_pk_str = cas_pk.to_string();

    // Are we in memory?
    let in_memory = ldb.cas().cache.memory_cache().get(&cas_pk_str).await;
    assert_eq!(Some(cas_value.clone()), in_memory);

    // Are we on disk?
    let on_disk_postcard = ldb
        .cas()
        .cache
        .disk_cache()
        .get(&cas_pk_str)
        .expect("cannot get from disk cache")
        .expect("cas pk not found in disk cache");
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
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let ldb = LayerDb::new(
        tempdir,
        setup_pg_db("cas_write_and_read_many").await,
        setup_nats_client(Some("cas_write_and_read_many".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");

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
    let tempdir = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let ldb = LayerDb::new(
        tempdir,
        setup_pg_db("cas_cold_read_from_db").await,
        setup_nats_client(Some("cas_cold_read_from_db".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");

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

    let cas_pk_str = cas_pk.to_string();

    // Delete from memory and disk
    ldb.cas().cache.memory_cache().remove(&cas_pk_str).await;
    let not_in_memory = ldb.cas().cache.memory_cache().get(&cas_pk_str).await;
    assert_eq!(not_in_memory, None);

    ldb.cas()
        .cache
        .disk_cache()
        .remove(&cas_pk_str)
        .expect("cannot remove from disk");
    let not_on_disk = ldb
        .cas()
        .cache
        .disk_cache()
        .get(&cas_pk_str)
        .expect("cannot get from disk cache");
    assert_eq!(not_on_disk, None);

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
        .get(&cas_pk_str)
        .expect("cannot get from disk cache")
        .expect("cas pk not found in disk cache");
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
    let tempdir_slash = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_axl = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let db = setup_pg_db("cas_writes_are_gossiped").await;

    // First, we need a layerdb for slash
    let ldb_slash = LayerDb::new(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("cas_writes_are_gossiped".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");

    // Then, we need a layerdb for axl
    let ldb_axl = LayerDb::new(
        tempdir_axl,
        db,
        setup_nats_client(Some("cas_write_to_db".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");

    let cas_value: Arc<CasValue> = Arc::new(serde_json::json!("stone sour").into());
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

    let cas_pk_str = cas_pk.to_string();

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
            .get(&cas_pk_str)
            .expect("cannot get from disk cache")
        {
            Some(on_disk_postcard) => {
                let on_disk: CasValue =
                    postcard::from_bytes(&on_disk_postcard[..]).expect("cannot deserialize data");
                assert_eq!(cas_value.as_ref(), &on_disk);
                break;
            }
            None => {
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
