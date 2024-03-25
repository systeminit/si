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
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::integration_test::{setup_nats_client, setup_pg_db};

mod cas;

type TestLayerDb = LayerDb<Arc<String>, String>;

#[tokio::test]
async fn activities() {
    let token = CancellationToken::new();

    let tempdir_slash = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_axl = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
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
    let token = CancellationToken::new();

    let tempdir_slash = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_axl = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
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

#[tokio::test]
async fn subscribe_rebaser_requests_work_queue() {
    let token = CancellationToken::new();

    let tempdir_slash = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_axl = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_duff = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let db = setup_pg_db("subscribe_rebaser_requests_work_queue").await;

    // we need a layerdb for slash, which will be a consumer of our work queue
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("subscribe_rebaser_requests_work_queue".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // we need a layerdb for axl, who will also be a consumer for our work queue
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db.clone(),
        setup_nats_client(Some("subscribe_rebaser_requests_work_queue".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    // we need a layerdb for duff, who will also be a consumer for our work queue
    let (ldb_duff, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_duff,
        db,
        setup_nats_client(Some("subscribe_rebaser_requests_work_queue".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_duff.pg_migrate().await.expect("migrate layerdb");

    // Subscribe to a work queue of rebase activities on axl and slash
    let mut axl_work_queue = ldb_axl
        .subscribe_rebaser_requests_work_queue()
        .await
        .expect("cannot retrieve a work queue");
    let mut slash_work_queue = ldb_slash
        .subscribe_rebaser_requests_work_queue()
        .await
        .expect("cannot retrieve a work queue");

    // Send a rebase request activity from duff
    let rebase_request = RebaseRequest::new(Ulid::new(), Ulid::new(), Ulid::new());
    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);
    let rebase_request_activity = Activity::rebase(rebase_request, metadata);
    // Publish an activity
    ldb_duff
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
    ldb_duff
        .publish_activity(&rebase_finished_activity)
        .await
        .expect("cannot publish activity");

    let which = tokio::select! {
        maybe_result = slash_work_queue.next() => {
            let request = maybe_result.expect("had no messages").expect("cannot retrieve the ack rebase request");
            assert_eq!(request.payload, rebase_request);
            request.ack().await.expect("cannot ack message");
            "slash".to_string()
        },
        maybe_result = axl_work_queue.next() => {
            let request = maybe_result.expect("had no messages").expect("cannot retrieve the ack rebase request");
            assert_eq!(request.payload, rebase_request);
            request.ack().await.expect("cannot ack message");
            "axl".to_string()
        },
    };

    // This is long enough to confirm that we get once-and-only-once delivery.
    // It isn't long enough to confirm that we didn't ack the payload, but that
    // is totally fine - we don't need to test that NATS works as directed.
    let sleep = tokio::time::sleep(tokio::time::Duration::from_millis(100));
    tokio::pin!(sleep);

    if which == "slash" {
        tokio::select! {
            maybe_result = axl_work_queue.next() => {
                assert!(maybe_result.is_none(), "expected no work, but there is some work to do");
            },
            _ = &mut sleep => {
            }
        }
    } else {
        tokio::select! {
            maybe_result = slash_work_queue.next() => {
                assert!(maybe_result.is_none(), "expected no work, but there is some work to do");
            },
            _ = &mut sleep => {
            }
        }
    }
}
