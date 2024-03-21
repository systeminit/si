mod cas;

use std::sync::Arc;

use futures::StreamExt;
use si_events::{Actor, ChangeSetId, Tenancy, WorkspacePk};
use si_layer_cache::{
    activities::{
        rebase::{RebaseFinished, RebaseRequest},
        Activity, ActivityPayloadDiscriminants,
    },
    event::LayeredEventMetadata,
    LayerDb,
};
use ulid::Ulid;

use crate::integration_test::{setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<Arc<String>, String>;

#[tokio::test]
async fn activities() {
    let tempdir_slash = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_axl = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let db = setup_pg_db("activities").await;

    // First, we need a layerdb for slash
    let ldb_slash: LayerDb<Arc<String>, String> = LayerDb::new(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("activities".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // Then, we need a layerdb for axl
    let ldb_axl: TestLayerDb = LayerDb::new(
        tempdir_axl,
        db,
        setup_nats_client(Some("activities".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    // Subscribe to all activities
    let mut activities = ldb_axl
        .subscribe_all_activities()
        .await
        .expect("cannot subscribe to all activities");

    let rebase_request = RebaseRequest::new(Ulid::new(), Ulid::new(), Ulid::new());
    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);
    let activity = Activity::rebase(rebase_request, metadata);
    // Publish an activity
    ldb_slash
        .publish_activity(&activity)
        .await
        .expect("cannot publish activity");

    let (restored_activity, _acker) = activities
        .next()
        .await
        .expect("no message waiting")
        .expect("error receiving message")
        .into_parts();
    assert_eq!(activity, restored_activity);
}

#[tokio::test]
async fn activities_subscribe_partial() {
    let tempdir_slash = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_axl = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let db = setup_pg_db("activities_subscribe_partial").await;

    // First, we need a layerdb for slash
    let ldb_slash: TestLayerDb = LayerDb::new(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("activities_subscribe_partial".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // Then, we need a layerdb for axl
    let ldb_axl: TestLayerDb = LayerDb::new(
        tempdir_axl,
        db,
        setup_nats_client(Some("activities_subscribe_partial".to_string())).await,
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    // Subscribe to only rebase finished activities
    let mut activities = ldb_axl
        .subscribe_activities(vec![ActivityPayloadDiscriminants::RebaseFinished])
        .await
        .expect("cannot subscribe to all activities");

    // Send a rebase request activity
    let rebase_request = RebaseRequest::new(Ulid::new(), Ulid::new(), Ulid::new());
    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);
    let rebase_request_activity = Activity::rebase(rebase_request, metadata);
    // Publish an activity
    ldb_slash
        .publish_activity(&rebase_request_activity)
        .await
        .expect("cannot publish activity");

    // Send a rebase finished activity
    let rebase_finished = RebaseFinished::new(
        si_layer_cache::activities::rebase::RebaseStatus::Error {
            message: "poop".to_string(),
        },
        Ulid::new(),
        Ulid::new(),
    );
    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);
    let rebase_finished_activity = Activity::rebase_finished(rebase_finished, metadata);
    // Publish an activity
    ldb_slash
        .publish_activity(&rebase_finished_activity)
        .await
        .expect("cannot publish activity");

    // The nats publishing rules would require that the first activity (the rebase request) be
    // recieved before the second (the rebase finished event). So we can confirm we have subject
    // filtering working.
    let (restored_activity, _acker) = activities
        .next()
        .await
        .expect("no message waiting")
        .expect("error receiving message")
        .into_parts();
    assert_eq!(rebase_finished_activity, restored_activity);
}
