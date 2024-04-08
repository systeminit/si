mod rebase;

use std::sync::Arc;

use futures::StreamExt;
use si_events::{Actor, ChangeSetId, Tenancy, WorkspacePk};
use si_layer_cache::{
    activities::ActivityPayloadDiscriminants, event::LayeredEventMetadata, LayerDb,
};
use tokio_util::sync::CancellationToken;

use crate::integration_test::{disk_cache_path, setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<Arc<String>, Arc<String>, String, String>;

#[tokio::test]
async fn activities() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");

    let db = setup_pg_db("activities").await;

    // First, we need a layerdb for slash
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("activities".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // Then, we need a layerdb for axl
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db,
        setup_nats_client(Some("activities".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    // Create our metadata
    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);

    // Subscribe to all activities
    let mut activities = ldb_axl
        .activity()
        .subscribe_all()
        .await
        .expect("cannot subscribe to all activities");

    // Publish an activity
    let activity = ldb_slash
        .activity()
        .test()
        .integration_test("drop me the bomb", metadata, None)
        .await
        .expect("cannot publish activity");

    let restored_activity = activities
        .next()
        .await
        .expect("no message waiting when one was expected")
        .expect("error receiving message");
    assert_eq!(activity, restored_activity);
}

#[tokio::test]
async fn activities_subscribe_partial() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");
    let db = setup_pg_db("activities_subscribe_partial").await;

    // First, we need a layerdb for slash
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("activities_subscribe_partial".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // Then, we need a layerdb for axl
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db,
        setup_nats_client(Some("activities_subscribe_partial".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    // Subscribe to only rebase finished activities
    let mut activities = ldb_axl
        .activity()
        .subscribe(vec![ActivityPayloadDiscriminants::IntegrationTestAlt])
        .await
        .expect("cannot subscribe to all activities");

    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);

    // Publish an activity
    ldb_slash
        .activity()
        .test()
        .integration_test("skid row", metadata.clone(), None)
        .await
        .expect("cannot publish activity");

    // Publish an activity
    let activity = ldb_slash
        .activity()
        .test()
        .integration_test_alt("kix", metadata, None)
        .await
        .expect("cannot publish activity");

    // The nats publishing rules would require that the first activity (the rebase request) be
    // recieved before the second (the rebase finished event). So we can confirm we have subject
    // filtering working.
    let restored_activity = activities
        .next()
        .await
        .expect("no message waiting")
        .expect("error receiving message");
    assert_eq!(activity, restored_activity);
}
