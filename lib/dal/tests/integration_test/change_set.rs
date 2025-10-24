use std::collections::HashSet;

use dal::{
    AccessBuilder,
    ChangeSet,
    ChangeSetStatus,
    Component,
    DalContext,
    DalContextBuilder,
    RequestContext,
    Workspace,
    WorkspacePk,
    change_set::view::OpenChangeSetsView,
    context::TransactionsErrorDiscriminants,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        create_user,
    },
    test,
};
use itertools::Itertools;
use pretty_assertions_sorted::assert_eq;
use si_db::HistoryActor;
use si_events::{
    AuthenticationMethod,
    AuthenticationMethodRole,
    authentication_method::AuthenticationMethodV1,
};

mod approval;

#[test]
async fn open_change_sets(ctx: &mut DalContext) {
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of open change sets exist.
    assert_eq!(
        2,                      // expected
        view.change_sets.len()  // actual
    );

    // Check that we collected "head" properly.
    let head_change_set_id = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("could not get the default change set id for the workspace");
    assert_eq!(
        head_change_set_id,      // expected
        view.head_change_set_id  // actual
    );

    // Ensure that the current change set is not "head".
    let current_change_set_id = ctx.change_set_id();
    assert_ne!(current_change_set_id, head_change_set_id);

    // Ensure that the views contain the change sets that we expect.
    let change_set_ids = HashSet::from_iter(view.change_sets.iter().map(|c| c.id));
    assert_eq!(
        HashSet::from([current_change_set_id, head_change_set_id]), // expected
        change_set_ids,                                             // actual
    );

    // Apply the change set and perform a blocking commit.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Assemble the view again and ensure only "head" exists.
    let mut view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of open change sets exist. There should only be one.
    let head_change_set_view = view.change_sets.pop().expect("change sets are empty");
    assert!(view.change_sets.is_empty());
    assert_eq!(
        view.head_change_set_id, // expected
        head_change_set_view.id  // actual
    );
    assert_eq!(
        head_change_set_id,      // expected
        head_change_set_view.id  // actual
    );

    // Create a new change set off HEAD.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork change set");

    // List views again.
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of open change sets exist... again.
    assert_eq!(
        2,                      // expected
        view.change_sets.len()  // actual
    );

    // Check that we collected "head" properly... again.
    let head_change_set_id_again = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("could not get the default change set id for the workspace");
    assert_eq!(
        head_change_set_id,       // expected
        head_change_set_id_again  // actual
    );
    assert_eq!(
        head_change_set_id,      // expected
        view.head_change_set_id  // actual
    );

    // Ensure that the current change set is not "head"... again.
    let current_change_set_id_again = ctx.change_set_id();
    assert_ne!(head_change_set_id, current_change_set_id_again);
    assert_ne!(current_change_set_id, current_change_set_id_again);

    // Ensure that the views contain the change sets that we expect... again.
    let change_set_ids_again = HashSet::from_iter(view.change_sets.iter().map(|c| c.id));
    assert_eq!(
        HashSet::from([current_change_set_id_again, head_change_set_id_again]), // expected
        change_set_ids_again,                                                   // actual
    );
}

pub const TEST_JWT_AUTHENTICATION_METHOD: AuthenticationMethod = AuthenticationMethodV1::Jwt {
    role: AuthenticationMethodRole::Web,
    token_id: None,
};

#[test]
async fn abandon_change_set_and_check_open_change_sets(ctx: &mut DalContext) {
    let change_set_name = "for abandonment".to_string();
    ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, &change_set_name)
        .await
        .expect("could not fork change set");

    // List open changesets.
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that the expected number of change sets exist....
    assert_eq!(
        3,                      // expected
        view.change_sets.len()  // actual
    );

    ChangeSetTestHelpers::abandon_change_set(ctx)
        .await
        .expect("could not abandon change set");

    // relist the open changesets.
    let view = OpenChangeSetsView::assemble(ctx)
        .await
        .expect("could not assemble view");

    // Check that we no longer have the abandoned changeset
    assert_eq!(
        2,                      // expected
        view.change_sets.len()  // actual
    );

    let change_set_names = Vec::from_iter(view.change_sets.iter().map(|c| c.name.clone()));
    assert!(!change_set_names.contains(&change_set_name))
}

#[test]
async fn build_from_request_context_limits_to_workspaces_user_has_access_to(
    ctx: &mut DalContext,
    ctx_builder: DalContextBuilder,
) {
    let user_1 = create_user(ctx).await.expect("Unable to create user");
    let user_2 = create_user(ctx).await.expect("Unable to create user");
    let user_1_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_1 workspace", "token")
            .await
            .expect("Unable to create workspace");
    let user_2_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_2 workspace", "token")
            .await
            .expect("Unable to create workspace");
    user_1
        .associate_workspace(ctx, *user_1_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    user_2
        .associate_workspace(ctx, *user_2_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to set up test data");

    let request_context = RequestContext {
        tenancy: si_db::Tenancy::new(*user_2_workspace.pk()),
        visibility: si_db::Visibility {
            change_set_id: user_2_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_1.pk()),
        request_ulid: None,
        authentication_method: TEST_JWT_AUTHENTICATION_METHOD,
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(
        builder_result
            .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into())
    );

    let request_context = RequestContext {
        tenancy: si_db::Tenancy::new(*user_1_workspace.pk()),
        visibility: si_db::Visibility {
            change_set_id: user_1_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_2.pk()),
        request_ulid: None,
        authentication_method: TEST_JWT_AUTHENTICATION_METHOD,
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(
        builder_result
            .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into())
    );
}

#[test]
async fn build_from_request_context_limits_to_change_sets_of_current_workspace(
    ctx: &mut DalContext,
    ctx_builder: DalContextBuilder,
) {
    let user_1 = create_user(ctx).await.expect("Unable to create user");
    let user_2 = create_user(ctx).await.expect("Unable to create user");
    let user_1_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_1 workspace", "token")
            .await
            .expect("Unable to create workspace");
    let user_2_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_2 workspace", "token")
            .await
            .expect("Unable to create workspace");
    user_1
        .associate_workspace(ctx, *user_1_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    user_2
        .associate_workspace(ctx, *user_2_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to set up test data");

    let request_context = RequestContext {
        tenancy: si_db::Tenancy::new(*user_1_workspace.pk()),
        visibility: si_db::Visibility {
            change_set_id: user_2_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_1.pk()),
        request_ulid: None,
        authentication_method: TEST_JWT_AUTHENTICATION_METHOD,
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(
        builder_result
            .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into())
    );

    let request_context = RequestContext {
        tenancy: si_db::Tenancy::new(*user_2_workspace.pk()),
        visibility: si_db::Visibility {
            change_set_id: user_1_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_2.pk()),
        request_ulid: None,
        authentication_method: TEST_JWT_AUTHENTICATION_METHOD,
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(
        builder_result
            .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into())
    );
}

#[test]
async fn cannot_find_change_set_across_workspaces(
    ctx: &mut DalContext,
    ctx_builder: DalContextBuilder,
) {
    let user_1 = create_user(ctx).await.expect("Unable to create user");
    let user_2 = create_user(ctx).await.expect("Unable to create user");
    let user_1_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_1 workspace", "token")
            .await
            .expect("Unable to create workspace");
    let user_2_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_2 workspace", "token")
            .await
            .expect("Unable to create workspace");
    user_1
        .associate_workspace(ctx, *user_1_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    user_2
        .associate_workspace(ctx, *user_2_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to set up test data");

    let request_context = RequestContext {
        tenancy: si_db::Tenancy::new(*user_2_workspace.pk()),
        visibility: si_db::Visibility {
            change_set_id: user_2_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_2.pk()),
        request_ulid: None,
        authentication_method: TEST_JWT_AUTHENTICATION_METHOD,
    };

    let mut user_2_dal_ctx = ctx_builder
        .build(request_context)
        .await
        .expect("built dal ctx for user 2");

    //create a new change set for user 2
    let user_2_change_set = ChangeSet::fork_head(&user_2_dal_ctx, "user 2")
        .await
        .expect("could not create change set");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(&mut user_2_dal_ctx)
        .await
        .expect("Unable to set up test data");

    let user_1_tenancy = si_db::Tenancy::new(*user_1_workspace.pk());
    let access_builder = AccessBuilder::new(
        user_1_tenancy,
        HistoryActor::User(user_1.pk()),
        None,
        TEST_JWT_AUTHENTICATION_METHOD,
    );

    let user_1_dal_context = ctx_builder
        .build_head(access_builder)
        .await
        .expect("could not build dal context");

    //first, let's ensure we can't find it when using the user 1 dal ctx
    let user_2_change_set_unfound = ChangeSet::find(&user_1_dal_context, user_2_change_set.id)
        .await
        .expect("could not find change set");
    assert!(user_2_change_set_unfound.is_none());

    // But if we search for the change set across all workspaces, we find it
    ChangeSet::get_by_id_across_workspaces(&user_1_dal_context, user_2_change_set.id)
        .await
        .expect("could not find change set");
}

#[test]
async fn build_from_request_context_allows_change_set_from_workspace_with_access(
    ctx: &mut DalContext,
    ctx_builder: DalContextBuilder,
) {
    let user_1 = create_user(ctx).await.expect("Unable to create user");
    let user_2 = create_user(ctx).await.expect("Unable to create user");
    let user_1_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_1 workspace", "token")
            .await
            .expect("Unable to create workspace");
    let user_2_workspace =
        Workspace::new_from_builtin(ctx, WorkspacePk::generate(), "user_2 workspace", "token")
            .await
            .expect("Unable to create workspace");
    user_1
        .associate_workspace(ctx, *user_1_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    user_2
        .associate_workspace(ctx, *user_2_workspace.pk())
        .await
        .expect("Unable to associate user with workspace");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to set up test data");

    let request_context = RequestContext {
        tenancy: si_db::Tenancy::new(*user_1_workspace.pk()),
        visibility: si_db::Visibility {
            change_set_id: user_1_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_1.pk()),
        request_ulid: None,
        authentication_method: TEST_JWT_AUTHENTICATION_METHOD,
    };

    let builder_result = ctx_builder.build(request_context).await;
    if let Err(e) = &builder_result {
        dbg!(e);
    }
    assert!(builder_result.is_ok());

    let request_context = RequestContext {
        tenancy: si_db::Tenancy::new(*user_2_workspace.pk()),
        visibility: si_db::Visibility {
            change_set_id: user_2_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_2.pk()),
        request_ulid: None,
        authentication_method: TEST_JWT_AUTHENTICATION_METHOD,
    };

    let builder_result = ctx_builder.build(request_context).await;
    if let Err(e) = &builder_result {
        dbg!(e);
    }
    assert!(builder_result.is_ok());
}

#[test]
async fn change_set_approval_flow(ctx: &mut DalContext) {
    // create a new change set
    let new_change_set = ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");
    let current_user = ChangeSet::extract_userid_from_context(ctx).await;

    // do something in it
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "small")
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    // request approval
    let mut change_set = ChangeSet::get_by_id(ctx, new_change_set.id)
        .await
        .expect("could not find change set");

    change_set
        .request_change_set_approval(ctx)
        .await
        .expect("could not request approval");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    let mut change_set = ChangeSet::get_by_id(ctx, new_change_set.id)
        .await
        .expect("could not find change set");

    // make sure everything looks right
    assert_eq!(change_set.status, ChangeSetStatus::NeedsApproval);
    assert!(change_set.merge_requested_at.is_some());
    assert_eq!(change_set.merge_requested_by_user_id, current_user);
    assert_eq!(change_set.reviewed_at, None);
    assert_eq!(change_set.reviewed_by_user_id, None);

    // let's reject it
    change_set
        .reject_change_set_for_apply(ctx)
        .await
        .expect("could not reject change set");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    let mut change_set = ChangeSet::get_by_id(ctx, new_change_set.id)
        .await
        .expect("could not find change set");
    assert_eq!(change_set.status, ChangeSetStatus::Rejected);
    assert!(change_set.merge_requested_at.is_some());
    assert_eq!(change_set.merge_requested_by_user_id, current_user);
    assert!(change_set.reviewed_at.is_some());
    assert_eq!(change_set.reviewed_by_user_id, current_user);

    // let's see if we can apply now, it should fail because the change set has not been approved
    let apply_result = ChangeSetTestHelpers::apply_change_set_to_base_approvals(ctx).await;
    assert!(apply_result.is_err());

    // now let's re-open it
    change_set
        .reopen_change_set(ctx)
        .await
        .expect("could not update status");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    let mut change_set = ChangeSet::get_by_id(ctx, new_change_set.id)
        .await
        .expect("could not find change set");
    assert_eq!(change_set.status, ChangeSetStatus::Open);
    assert_eq!(change_set.merge_requested_at, None);
    assert_eq!(change_set.merge_requested_by_user_id, None);
    assert_eq!(change_set.reviewed_at, None);
    assert_eq!(change_set.reviewed_by_user_id, None);

    // now let's request approval again
    change_set
        .request_change_set_approval(ctx)
        .await
        .expect("could not request approval");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    let mut change_set = ChangeSet::get_by_id(ctx, new_change_set.id)
        .await
        .expect("could not find change set");

    // make sure everything looks right
    assert_eq!(change_set.status, ChangeSetStatus::NeedsApproval);
    assert!(change_set.merge_requested_at.is_some());
    assert_eq!(change_set.merge_requested_by_user_id, current_user);
    assert_eq!(change_set.reviewed_at, None);
    assert_eq!(change_set.reviewed_by_user_id, None);

    // this time we will approve
    change_set
        .approve_change_set_for_apply(ctx)
        .await
        .expect("could not approve");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    let change_set = ChangeSet::get_by_id(ctx, new_change_set.id)
        .await
        .expect("could not find change set");

    // make sure everything looks right
    assert_eq!(change_set.status, ChangeSetStatus::Approved);
    assert!(change_set.merge_requested_at.is_some());
    assert_eq!(change_set.merge_requested_by_user_id, current_user);
    assert!(change_set.reviewed_at.is_some());
    assert_eq!(change_set.reviewed_by_user_id, current_user);

    // now let's apply it!

    ChangeSetTestHelpers::apply_change_set_to_base_approvals(ctx)
        .await
        .expect("could not apply to head");

    // should have one component
    let mut components = Component::list(ctx)
        .await
        .expect("could not list components");
    assert_eq!(components.len(), 1);
    let only_component = components.pop().expect("has one in there");
    assert_eq!(only_component.id(), component.id());

    // now let's create another change set and ensure force_apply works as expected
    let _new_change_set = ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");

    // do something in it
    let second_component =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "small")
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    // force apply
    ChangeSetTestHelpers::force_apply_change_set_to_base_approvals(ctx)
        .await
        .expect("could not force apply change set");
    // should have two components now
    let components = Component::list(ctx)
        .await
        .expect("could not list components");
    assert_eq!(components.len(), 2);
    let component_ids = [component.id(), second_component.id()];
    let components = components
        .into_iter()
        .filter(|comp| component_ids.contains(&comp.id()))
        .collect_vec();
    assert_eq!(components.len(), 2);
}

#[test]
async fn update_pointer_tracks_last_used(ctx: &mut DalContext) {
    // Create change set with initial snapshot
    let mut change_set = ChangeSet::fork_head(ctx, "test-tracking")
        .await
        .expect("could not fork head");
    let old_snapshot = change_set.workspace_snapshot_address;

    // Create a new snapshot to update to
    let new_snapshot_address = ctx
        .workspace_snapshot()
        .expect("could not get workspace snapshot")
        .write(ctx)
        .await
        .expect("could not write snapshot");

    // Update pointer
    change_set
        .update_pointer(ctx, new_snapshot_address)
        .await
        .expect("could not update pointer");

    // Verify old snapshot tracked in snapshot_last_used
    let row = ctx
        .txns()
        .await
        .expect("could not get txns")
        .pg()
        .query_opt(
            "SELECT snapshot_id, last_used_at FROM snapshot_last_used WHERE snapshot_id = $1",
            &[&old_snapshot.to_string()],
        )
        .await
        .expect("could not query snapshot_last_used");

    assert!(row.is_some(), "Old snapshot should be tracked");

    let row = row.unwrap();
    let snapshot_id: String = row
        .try_get("snapshot_id")
        .expect("could not get snapshot_id");
    assert_eq!(snapshot_id, old_snapshot.to_string());
}
