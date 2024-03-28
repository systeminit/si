use std::sync::Arc;

use futures::StreamExt;
use si_events::{Actor, ChangeSetId, Tenancy, WorkspacePk, WorkspaceSnapshotAddress};
use si_layer_cache::{activities::ActivityId, event::LayeredEventMetadata, LayerDb};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::integration_test::{setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<Arc<String>, String>;

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
        .activity()
        .rebase()
        .subscribe_work_queue()
        .await
        .expect("cannot retrieve a work queue");
    let mut slash_work_queue = ldb_slash
        .activity()
        .rebase()
        .subscribe_work_queue()
        .await
        .expect("cannot retrieve a work queue");

    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);

    let rebase_request_activity = ldb_duff
        .activity()
        .rebase()
        .rebase(
            Ulid::new(),
            WorkspaceSnapshotAddress::new(b"poop"),
            Ulid::new(),
            metadata.clone(),
        )
        .await
        .expect("cannot publish rebase request");

    // Send a rebase finished activity
    let _rebase_finished_activity = ldb_duff
        .activity()
        .rebase()
        .finished(
            si_layer_cache::activities::rebase::RebaseStatus::Error {
                message: "poop".to_string(),
            },
            Ulid::new(),
            WorkspaceSnapshotAddress::new(b"skid row"),
            metadata,
            ActivityId::new(),
        )
        .await
        .expect("cannot send rebase finished");

    let which = tokio::select! {
        maybe_result = slash_work_queue.next() => {
            let request = maybe_result.expect("had no messages").expect("cannot retrieve the ack rebase request");
            assert_eq!(request.id, rebase_request_activity.id);
            request.ack().await.expect("cannot ack message");
            "slash".to_string()
        },
        maybe_result = axl_work_queue.next() => {
            let request = maybe_result.expect("had no messages").expect("cannot retrieve the ack rebase request");
            assert_eq!(request.id, rebase_request_activity.id);
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

#[tokio::test]
async fn rebase_and_wait() {
    let token = CancellationToken::new();

    let tempdir_slash = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let tempdir_axl = tempfile::TempDir::new_in("/tmp").expect("cannot create tempdir");
    let db = setup_pg_db("rebase_and_wait").await;

    // we need a layerdb for slash, who will send the rebase request
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("rebase_and_wait".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // we need a layerdb for axl, who will send the reply
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db.clone(),
        setup_nats_client(Some("rebase_and_wait".to_string())).await,
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    // Subscribe to a work queue of rebase activities on axl
    let mut axl_work_queue = ldb_axl
        .activity()
        .rebase()
        .subscribe_work_queue()
        .await
        .expect("cannot retrieve a work queue");

    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);
    let metadata_for_task = metadata.clone();

    let rebase_request_task = tokio::spawn(async move {
        ldb_slash
            .activity()
            .rebase()
            .rebase_and_wait(
                Ulid::new(),
                WorkspaceSnapshotAddress::new(b"poop"),
                Ulid::new(),
                metadata_for_task,
            )
            .await
    });

    let rebase_request = axl_work_queue
        .next()
        .await
        .expect("should have an message, but the channel is closed")
        .expect("should have a rebase request, but we have an error");
    rebase_request.ack().await.expect("cannot ack the message");

    // Send a rebase finished activity
    let rebase_finished_activity = ldb_axl
        .activity()
        .rebase()
        .finished(
            si_layer_cache::activities::rebase::RebaseStatus::Error {
                message: "poop".to_string(),
            },
            Ulid::new(),
            WorkspaceSnapshotAddress::new(b"skid row"),
            metadata,
            rebase_request.id,
        )
        .await
        .expect("cannot send rebase finished");

    let received_finish_activity = rebase_request_task
        .await
        .expect("rebase request task failed")
        .expect("expected rebase finished activity, but got an error");

    assert_eq!(received_finish_activity, rebase_finished_activity);
}
