use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use si_events::{Actor, ChangeSetId, Tenancy, WorkspacePk, WorkspaceSnapshotAddress};
use si_layer_cache::{
    activities::ActivityId, event::LayeredEventMetadata, memory_cache::MemoryCacheConfig, LayerDb,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::integration_test::{disk_cache_path, setup_nats_client, setup_pg_db};

type TestLayerDb = LayerDb<Arc<String>, Arc<String>, String>;

#[tokio::test]
async fn subscribe_rebaser_requests_work_queue() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");
    let tempdir_duff = disk_cache_path(&tempdir, "duff");
    let db = setup_pg_db("subscribe_rebaser_requests_work_queue").await;

    // we need a layerdb for slash, which will be a consumer of our work queue
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("subscribe_rebaser_requests_work_queue".to_string())).await,
        MemoryCacheConfig::default(),
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
        MemoryCacheConfig::default(),
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
        MemoryCacheConfig::default(),
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
        maybe_result = slash_work_queue.recv() => {
            let request = maybe_result.expect("had no messages");
            assert_eq!(request.id, rebase_request_activity.id);
            "slash".to_string()
        },
        maybe_result = axl_work_queue.recv() => {
            let request = maybe_result.expect("had no messages");
            assert_eq!(request.id, rebase_request_activity.id);
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
            maybe_result = axl_work_queue.recv() => {
                assert!(maybe_result.is_none(), "expected no work, but there is some work to do");
            },
            _ = &mut sleep => {
            }
        }
    } else {
        tokio::select! {
            maybe_result = slash_work_queue.recv() => {
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

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");

    let db = setup_pg_db("rebase_and_wait").await;

    // we need a layerdb for slash, who will send the rebase request
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("rebase_and_wait".to_string())).await,
        MemoryCacheConfig::default(),
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
        MemoryCacheConfig::default(),
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
        .recv()
        .await
        .expect("should have an message, but the channel is closed");

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

#[tokio::test(flavor = "multi_thread")]
async fn rebase_requests_work_queue_stress() {
    let token = CancellationToken::new();

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");
    let tempdir_duff = disk_cache_path(&tempdir, "duff");
    let db = setup_pg_db("rebase_requests_work_queue_stress").await;

    // we need a layerdb for slash, which will be a consumer of our work queue
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("rebase_requests_work_queue_stress".to_string())).await,
        MemoryCacheConfig::default(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // we need a layerdb for axl, who will also be a consumer for our work queue
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db.clone(),
        setup_nats_client(Some("rebase_requests_work_queue_stress".to_string())).await,
        MemoryCacheConfig::default(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    // we need a layerdb for duff, who will also be a producer for our work queue
    let (ldb_duff, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_duff,
        db,
        setup_nats_client(Some("rebase_requests_work_queue_stress".to_string())).await,
        MemoryCacheConfig::default(),
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
    let send_meta = metadata.clone();

    let rebase_activities = 10_000;

    static MESSAGE_COUNTER: AtomicI32 = AtomicI32::new(0);

    let tracker = TaskTracker::new();

    let axl_process_token = token.clone();
    let axl_process_handle = tracker.spawn(async move {
        while let Some(_request) = axl_work_queue.recv().await {
            MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
            if axl_process_token.is_cancelled() {
                break;
            }
        }
    });
    let slash_process_token = token.clone();
    let slash_process_handle = tracker.spawn(async move {
        while let Some(_request) = slash_work_queue.recv().await {
            MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
            if slash_process_token.is_cancelled() {
                break;
            }
        }
    });

    let send_process_token = token.clone();
    tracker.spawn(async move {
        let mut count = 0;
        while count < rebase_activities {
            let _rebase_request_activity = ldb_duff
                .activity()
                .rebase()
                .rebase(
                    Ulid::new(),
                    WorkspaceSnapshotAddress::new(b"poop"),
                    Ulid::new(),
                    send_meta.clone(),
                )
                .await
                .expect("cannot publish rebase request");
            count += 1;
            if send_process_token.is_cancelled() {
                break;
            }
        }
    });

    let check_token = token.clone();
    let all_messages_processed_stream = tracker.spawn(async move {
        loop {
            let count = MESSAGE_COUNTER.load(Ordering::SeqCst);
            if count == rebase_activities {
                break;
            }
            if check_token.is_cancelled() {
                break;
            }
        }
    });
    let timeout_handle = tracker.spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    });
    tracker.close();

    let result = tokio::select!(
        _e = axl_process_handle => {
            token.cancel();
          "axl".to_string()
        },
        _e = slash_process_handle => {
            token.cancel();
          "slash".to_string()
        },
        _ = all_messages_processed_stream => {
            token.cancel();
           "finished".to_string()
        }
        _ = timeout_handle => {
            token.cancel();
           "deadline".to_string()
        },
        () = token.cancelled() => {
           "finished".to_string()
        }
    );
    match result.as_str() {
        "axl" => panic!("axl write process died"),
        "slash" => panic!("slash write process died"),
        "finished" => {}
        "deadline" => panic!("took longer than 10 seconds to process messages"),
        _ => panic!("I dunno what happened"),
    }
}

// This test ensures that the multiplexer behavior for activities is working correctly. It
// simulates sending thousands of rebase requests and responses, ensuring that multiple blocking
// waits all receive their replies.
#[tokio::test(flavor = "multi_thread")]
async fn rebase_and_wait_stress() {
    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    let tempdir = tempfile::TempDir::new().expect("cannot create tempdir");

    let tempdir_slash = disk_cache_path(&tempdir, "slash");
    let tempdir_axl = disk_cache_path(&tempdir, "axl");

    let db = setup_pg_db("rebase_and_wait_stress").await;

    // we need a layerdb for slash, who will send the rebase request
    let (ldb_slash, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_slash,
        db.clone(),
        setup_nats_client(Some("rebase_and_wait_stress".to_string())).await,
        MemoryCacheConfig::default(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_slash.pg_migrate().await.expect("migrate layerdb");

    // we need a layerdb for axl, who will send the reply
    let (ldb_axl, _): (TestLayerDb, _) = LayerDb::initialize(
        tempdir_axl,
        db.clone(),
        setup_nats_client(Some("rebase_and_wait_stress".to_string())).await,
        MemoryCacheConfig::default(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb_axl.pg_migrate().await.expect("migrate layerdb");

    let rebase_activities = 5_000;
    static SENT_REQUEST_COUNTER: AtomicI32 = AtomicI32::new(0);
    static SENT_REPLY_COUNTER: AtomicI32 = AtomicI32::new(0);
    static RECV_REPLY_COUNTER: AtomicI32 = AtomicI32::new(0);

    let tenancy = Tenancy::new(WorkspacePk::new(), ChangeSetId::new());
    let actor = Actor::System;
    let metadata = LayeredEventMetadata::new(tenancy, actor);
    let metadata_for_processor = metadata.clone();

    // Set up a processor, rouhgly equivalent to a rebaser
    let processor_handle = tracker.spawn(async move {
        // Subscribe to a work queue of rebase activities on axl
        let mut axl_work_queue = ldb_axl
            .activity()
            .rebase()
            .subscribe_work_queue()
            .await
            .expect("cannot retrieve a work queue");
        while let Some(rebase_request) = axl_work_queue.recv().await {
            let mp = metadata_for_processor.clone();
            let _rebase_finished_activity = ldb_axl
                .activity()
                .rebase()
                .finished(
                    si_layer_cache::activities::rebase::RebaseStatus::Error {
                        message: "poop".to_string(),
                    },
                    Ulid::new(),
                    WorkspaceSnapshotAddress::new(b"skid row"),
                    mp,
                    rebase_request.id,
                )
                .await
                .expect("cannot send rebase finished");
            SENT_REPLY_COUNTER.fetch_add(1, Ordering::Relaxed);
            //if SENT_REPLY_COUNTER.load(Ordering::SeqCst) == rebase_activities {
            //    break;
            //}
            //dbg!("recv reply {}", count);
        }
    });

    let mut rebase_waiter_handles = Vec::new();
    for _i in 0..11 {
        let metadata_for_sender = metadata.clone();
        let ldb_slash_clone = ldb_slash.clone();
        let rebase_waiter_handle = tracker.spawn(async move {
            loop {
                SENT_REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
                let mp = metadata_for_sender.clone();
                let _response = ldb_slash_clone
                    .activity()
                    .rebase()
                    .rebase_and_wait(
                        Ulid::new(),
                        WorkspaceSnapshotAddress::new(b"poop"),
                        Ulid::new(),
                        mp,
                    )
                    .await;
                RECV_REPLY_COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        });
        rebase_waiter_handles.push(rebase_waiter_handle);
    }

    let check_token = token.clone();
    let all_messages_processed_stream = tracker.spawn(async move {
        loop {
            let recv_reply_count = RECV_REPLY_COUNTER.load(Ordering::SeqCst);
            if recv_reply_count >= rebase_activities {
                break;
            }
            if check_token.is_cancelled() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    let timeout_handle = tracker.spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    });
    tracker.close();

    let result = tokio::select!(
        e = processor_handle => {
           println!("processor exited: {:?}", e);
            token.cancel();
          "processor".to_string()
        },
        e = futures::future::select_all(rebase_waiter_handles) => {
           println!("rebase_waiter exited: {:?}", e);
            token.cancel();
          "rebase_waiter".to_string()
        },
        _e = all_messages_processed_stream => {
            token.cancel();
           "finished".to_string()
        }
        _ = timeout_handle => {
            token.cancel();
           "deadline".to_string()
        },
        () = token.cancelled() => {
           "finished".to_string()
        }
    );

    match result.as_ref() {
        "processor" => panic!("Processing task has paniced"),
        "rebase_waiter" => panic!("Rebase send/wait task has paniced"),
        "finished" => {}
        "deadline" => panic!("test took longer than 10 seconds to complete"),
        _ => panic!("wtf"),
    }
}
