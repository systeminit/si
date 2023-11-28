use base64::{engine::general_purpose, Engine};
use content_store::ContentHash;
use dal::change_set_pointer::ChangeSetPointer;
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::workspace_snapshot::conflict::Conflict;
use dal::workspace_snapshot::content_address::ContentAddress;
use dal::workspace_snapshot::node_weight::NodeWeight;
use dal::workspace_snapshot::update::Update;
use dal::{
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncId, Tenancy, Visibility,
    Workspace, WorkspacePk, WorkspaceSnapshot,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use rebaser_client::Client;
use rebaser_core::ChangeSetReplyMessage;

#[test]
async fn modify_func_node(ctx: &mut DalContext) {
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is code");

    let func = Func::new(
        ctx,
        "test",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Boolean,
        None::<String>,
        Some(code_base64),
    )
    .await
    .expect("able to make a func");

    ctx.commit().await.expect("unable to commit");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    Func::get_by_id(ctx, func.id)
        .await
        .expect("able to get func by id");

    let new_code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is new code");

    let func = func
        .modify(ctx, |f| {
            f.code_base64 = Some(new_code_base64.clone());

            Ok(())
        })
        .await
        .expect("able to modify func");

    let conflicts = ctx.commit().await.expect("unable to commit");
    assert!(matches!(conflicts, None));

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity again");

    let modified_func = Func::get_by_id(ctx, func.id)
        .await
        .expect("able to get func by id again");

    assert_eq!(Some(new_code_base64), modified_func.code_base64);
}

#[test]
async fn func_node_with_arguments(ctx: &mut DalContext) {
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is code");

    let func = Func::new(
        ctx,
        "test",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Boolean,
        None::<String>,
        Some(code_base64),
    )
    .await
    .expect("able to make a func");

    ctx.commit().await.expect("unable to commit");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    Func::get_by_id(ctx, func.id)
        .await
        .expect("able to get func by id");

    let new_code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is new code");

    // modify func
    let func = func
        .modify(ctx, |f| {
            f.name = "test:modified".into();
            f.code_base64 = Some(new_code_base64.clone());

            Ok(())
        })
        .await
        .expect("able to modify func");

    // create func arguments
    let arg_1 = FuncArgument::new(ctx, "argle bargle", FuncArgumentKind::Object, None, func.id)
        .await
        .expect("able to create func argument");
    FuncArgument::new(ctx, "argy bargy", FuncArgumentKind::Object, None, func.id)
        .await
        .expect("able to create func argument 2");

    let conflicts = ctx.commit().await.expect("unable to commit");
    assert!(matches!(conflicts, None));

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity again");

    let modified_func = Func::get_by_id(ctx, func.id)
        .await
        .expect("able to get func by id again");

    assert_eq!(
        Some(new_code_base64).as_deref(),
        modified_func.code_base64.as_deref()
    );

    let args = FuncArgument::list_for_func(ctx, modified_func.id)
        .await
        .expect("able to list args");

    assert_eq!(2, args.len());

    // Modify func argument
    FuncArgument::modify_by_id(ctx, arg_1.id, |arg| {
        arg.name = "bargle argle".into();

        Ok(())
    })
    .await
    .expect("able to modify func");

    let conflicts = ctx.commit().await.expect("unable to commit");
    assert!(matches!(conflicts, None));

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity again");

    let func = Func::get_by_id(ctx, func.id)
        .await
        .expect("able to get func by id again");

    let args = FuncArgument::list_for_func(ctx, func.id)
        .await
        .expect("able to list args again");

    assert_eq!(2, args.len());

    let modified_arg = args
        .iter()
        .find(|a| a.id == arg_1.id)
        .expect("able to get modified func arg");

    assert_eq!("bargle argle", modified_arg.name.as_str());
}

#[test]
async fn delete_func_node(ctx: &mut DalContext) {
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is code");

    let func = Func::new(
        ctx,
        "test",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Boolean,
        None::<String>,
        Some(code_base64),
    )
    .await
    .expect("able to make a func");

    ctx.commit().await.expect("unable to commit");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let snapshot_id_before_deletion = {
        ctx.workspace_snapshot()
            .expect("get snap")
            .try_lock()
            .expect("lock snap")
            .id()
    };

    Func::get_by_id(ctx, func.id)
        .await
        .expect("able to get func by id");

    Func::remove(ctx, func.id).expect("able to remove func");

    assert!(Func::get_by_id(ctx, func.id).await.is_err());

    let conflicts = ctx.commit().await.expect("unable to commit");
    assert!(matches!(conflicts, None));

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let snapshot_id_after_deletion = {
        ctx.workspace_snapshot()
            .expect("get snap")
            .try_lock()
            .expect("lock snap")
            .id()
    };

    // A sanity check
    assert_ne!(snapshot_id_before_deletion, snapshot_id_after_deletion);

    let result = Func::get_by_id(ctx, func.id).await;
    assert!(result.is_err());
}

// #[test]
// async fn pure_update_and_single_conflict(ctx: &mut DalContext) {
//     ctx.update_visibility(Visibility::new_head(false));
//     ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

//     // Start with the base change set and the initial snapshot.
//     let mut base_change_set = ChangeSetPointer::new(ctx, "main")
//         .await
//         .expect("could not create change set");
//     let base_change_set = &mut base_change_set;
//     let mut snapshot = WorkspaceSnapshot::initial(ctx, &base_change_set)
//         .await
//         .expect("could not create workspace snapshot");

//     // Add a new node, write and update the pointer.
//     let olivia_rodrigo_id = base_change_set
//         .generate_ulid()
//         .expect("could not generate id");
//     let olivia_rodrigo_node_index = snapshot
//         .add_node(
//             NodeWeight::new_content(
//                 base_change_set,
//                 olivia_rodrigo_id,
//                 ContentAddress::Component(ContentHash::from("lacy - olivia rodrigo")),
//             )
//             .expect("could not create node weight"),
//         )
//         .expect("could not add node");
//     snapshot
//         .add_edge_from_root(base_change_set, olivia_rodrigo_node_index)
//         .expect("could not add edge");
//     snapshot
//         .write(ctx, base_change_set.vector_clock_id())
//         .await
//         .expect("could not write snapshot");
//     base_change_set
//         .update_pointer(ctx, snapshot.id())
//         .await
//         .expect("could not update change set");

//     // Create another change set and update the snapshot.
//     let mut forked_change_set = ChangeSetPointer::new(ctx, "fork")
//         .await
//         .expect("could not create change set");
//     let forked_change_set = &mut forked_change_set;
//     let mut forked_snapshot = WorkspaceSnapshot::find_for_change_set(ctx, base_change_set.id)
//         .await
//         .expect("could not find snapshot");
//     let victoria_monet_id = forked_change_set
//         .generate_ulid()
//         .expect("could not generate id");
//     let victoria_monet_node_index = forked_snapshot
//         .add_node(
//             NodeWeight::new_content(
//                 forked_change_set,
//                 victoria_monet_id,
//                 ContentAddress::Component(ContentHash::from("i'm the one - victoria monét")),
//             )
//             .expect("could not create node weight"),
//         )
//         .expect("could not add node");
//     let victoria_monet_edge_index = forked_snapshot
//         .add_edge_from_root(forked_change_set, victoria_monet_node_index)
//         .expect("could not add edge");
//     forked_snapshot
//         .write(ctx, forked_change_set.vector_clock_id())
//         .await
//         .expect("could not write snapshot");
//     forked_change_set
//         .update_pointer(ctx, forked_snapshot.id())
//         .await
//         .expect("could not update change set");

//     // Commit all changes made so that the rebaser can access them.
//     ctx.blocking_commit().await.expect("could not commit");

//     // Create a rebaser client and open a change set loop.
//     let mut client = Client::new().await.expect("could not build client");
//     let _ = client
//         .open_stream_for_change_set(base_change_set.id.into())
//         .await
//         .expect("could not send management");

//     // Cache expected updates and then perform a rebase.
//     let expected_updates = [Update::NewEdge {
//         source: snapshot.root().expect("could not get root"),
//         destination: forked_snapshot
//             .get_node_index_by_id(victoria_monet_id)
//             .expect("could not get node index"),
//         edge_weight: forked_snapshot
//             .get_edge_by_index_stableish(victoria_monet_edge_index)
//             .expect("could not find edge by index"),
//     }];
//     let response = client
//         .request_rebase(
//             base_change_set.id.into(),
//             forked_snapshot.id().into(),
//             forked_change_set.vector_clock_id().into(),
//         )
//         .await
//         .expect("could not send");

//     // Ensure the rebase was successful and no updates needed to be performed.
//     match response {
//         ChangeSetReplyMessage::Success { updates_performed } => {
//             let actual_updates: Vec<Update> =
//                 serde_json::from_value(updates_performed).expect("could not deserialize");
//             assert_eq!(
//                 &expected_updates,         // expected
//                 actual_updates.as_slice()  // actual
//             );
//         }
//         ChangeSetReplyMessage::ConflictsFound {
//             conflicts_found,
//             updates_found_and_skipped: _,
//         } => {
//             let conflicts: Vec<Conflict> =
//                 serde_json::from_value(conflicts_found).expect("could not deserialize");
//             panic!("unexpected conflicts: {conflicts:?}");
//         }
//         ChangeSetReplyMessage::Error { message } => {
//             panic!("unexpected error: {message}");
//         }
//     }

//     // Now, create a conflict.
//     let mut snapshot = WorkspaceSnapshot::find_for_change_set(ctx, base_change_set.id)
//         .await
//         .expect("could not find snapshot");
//     snapshot
//         .update_content(
//             base_change_set,
//             olivia_rodrigo_id,
//             ContentHash::from("onto updated"),
//         )
//         .expect("could not update content");
//     snapshot
//         .write(ctx, base_change_set.vector_clock_id())
//         .await
//         .expect("could not write snapshot");
//     base_change_set
//         .update_pointer(ctx, snapshot.id())
//         .await
//         .expect("could not update change set");
//     let mut forked_snapshot = WorkspaceSnapshot::find_for_change_set(ctx, forked_change_set.id)
//         .await
//         .expect("could not find snapshot");
//     forked_snapshot
//         .update_content(
//             forked_change_set,
//             olivia_rodrigo_id,
//             ContentHash::from("to rebase updated"),
//         )
//         .expect("could not update content");
//     forked_snapshot
//         .write(ctx, forked_change_set.vector_clock_id())
//         .await
//         .expect("could not write snapshot");
//     forked_change_set
//         .update_pointer(ctx, forked_snapshot.id())
//         .await
//         .expect("could not update change set");

//     // Commit all changes made so that the rebaser can access them.
//     ctx.blocking_commit().await.expect("could not commit");

//     // Cache the expected conflict and perform the rebase with the conflict.
//     let expected_conflicts = [Conflict::NodeContent {
//         onto: forked_snapshot
//             .get_node_index_by_id(olivia_rodrigo_id)
//             .expect("could not get node index by id"),
//         to_rebase: snapshot
//             .get_node_index_by_id(olivia_rodrigo_id)
//             .expect("could not get node index by id"),
//     }];
//     let response = client
//         .request_rebase(
//             base_change_set.id.into(),
//             forked_snapshot.id().into(),
//             forked_change_set.vector_clock_id().into(),
//         )
//         .await
//         .expect("could not send");

//     // Ensure we see the conflict.
//     match response {
//         ChangeSetReplyMessage::Success { updates_performed } => {
//             let updates_performed: Vec<Update> =
//                 serde_json::from_value(updates_performed).expect("could not deserialize");
//             panic!("unexpected success: {updates_performed:?}")
//         }
//         ChangeSetReplyMessage::ConflictsFound {
//             conflicts_found,
//             updates_found_and_skipped,
//         } => {
//             let actual_conflicts: Vec<Conflict> =
//                 serde_json::from_value(conflicts_found).expect("could not deserialize");
//             assert_eq!(
//                 &expected_conflicts,         // expected
//                 actual_conflicts.as_slice()  // actual
//             );
//             let updates_found_and_skipped: Vec<Update> =
//                 serde_json::from_value(updates_found_and_skipped).expect("could not deserialize");
//             assert!(updates_found_and_skipped.is_empty());
//         }
//         ChangeSetReplyMessage::Error { message } => {
//             panic!("unexpected error: {message}");
//         }
//     }

//     // TODO(nick): move cleanup to the test harness.
//     let _ = client
//         .close_stream_for_change_set(base_change_set.id.into())
//         .await;
// }
