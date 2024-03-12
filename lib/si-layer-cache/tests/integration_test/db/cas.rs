use std::sync::Arc;

use si_events::{Actor, CasPk, CasValue, ChangeSetPk, ContentHash, Tenancy, UserPk, WorkspacePk};
use si_layer_cache::{persister::PersistStatus, LayerDb};

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

    let cas_pk = CasPk::new(ContentHash::new(b"corey taylor"));
    let cas_value: Arc<CasValue> = Arc::new(serde_json::json!("stone sour").into());
    let status = ldb
        .cas()
        .write(
            cas_pk,
            cas_value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetPk::new()),
            Actor::User(UserPk::new()),
        )
        .await
        .expect("failed to write to layerdb");
    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Write failed; {e}"),
    }

    // Are we in memory?
    let in_memory = ldb.cas().cache.memory_cache().get(&cas_pk).await;
    assert_eq!(Some(cas_value.clone()), in_memory);

    // Are we on disk?
    let on_disk_postcard = ldb
        .cas()
        .cache
        .disk_cache()
        .get(&cas_pk)
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
        .get(&cas_pk)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: CasValue =
        postcard::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(cas_value.as_ref(), &in_pg);
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

    let cas_pk = CasPk::new(ContentHash::new(b"corey taylor"));
    let cas_value: Arc<CasValue> = Arc::new(serde_json::json!("stone sour").into());
    let status = ldb
        .cas()
        .write(
            cas_pk,
            cas_value.clone(),
            None,
            Tenancy::new(WorkspacePk::new(), ChangeSetPk::new()),
            Actor::User(UserPk::new()),
        )
        .await
        .expect("failed to write to layerdb");
    match status.get_status().await.expect("failed to get status") {
        PersistStatus::Finished => {}
        PersistStatus::Error(e) => panic!("Write failed; {e}"),
    }

    // Delete from memory and disk
    ldb.cas().cache.memory_cache().remove(&cas_pk).await;
    let not_in_memory = ldb.cas().cache.memory_cache().get(&cas_pk).await;
    assert_eq!(not_in_memory, None);

    ldb.cas()
        .cache
        .disk_cache()
        .remove(&cas_pk)
        .expect("cannot remove from disk");
    let not_on_disk = ldb
        .cas()
        .cache
        .disk_cache()
        .get(&cas_pk)
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
    let in_memory = ldb.cas().cache.memory_cache().get(&cas_pk).await;
    assert_eq!(Some(cas_value.clone()), in_memory);

    // Are we on disk?
    let on_disk_postcard = ldb
        .cas()
        .cache
        .disk_cache()
        .get(&cas_pk)
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
        .get(&cas_pk)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: CasValue =
        postcard::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(cas_value.as_ref(), &in_pg);
}
