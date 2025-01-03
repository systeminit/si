use dal::change_set::view::OpenChangeSetsView;
use dal::{
    context::TransactionsErrorDiscriminants, DalContext, DalContextBuilder, HistoryActor,
    RequestContext, Workspace, WorkspacePk,
};
use dal::{ChangeSet, ChangeSetStatus, Component};
use dal_test::helpers::{
    create_component_for_default_schema_name_in_default_view, create_user, ChangeSetTestHelpers,
};
use dal_test::test;
use itertools::Itertools;
use pretty_assertions_sorted::assert_eq;
use std::collections::HashSet;

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
        tenancy: dal::Tenancy::new(*user_2_workspace.pk()),
        visibility: dal::Visibility {
            change_set_id: user_2_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_1.pk()),
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(builder_result
        .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into()));

    let request_context = RequestContext {
        tenancy: dal::Tenancy::new(*user_1_workspace.pk()),
        visibility: dal::Visibility {
            change_set_id: user_1_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_2.pk()),
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(builder_result
        .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into()));
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
        tenancy: dal::Tenancy::new(*user_1_workspace.pk()),
        visibility: dal::Visibility {
            change_set_id: user_2_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_1.pk()),
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(builder_result
        .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into()));

    let request_context = RequestContext {
        tenancy: dal::Tenancy::new(*user_2_workspace.pk()),
        visibility: dal::Visibility {
            change_set_id: user_1_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_2.pk()),
    };

    let builder_result = ctx_builder.build(request_context).await;
    assert!(builder_result
        .is_err_and(|e| TransactionsErrorDiscriminants::BadWorkspaceAndChangeSet == e.into()));
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
        tenancy: dal::Tenancy::new(*user_1_workspace.pk()),
        visibility: dal::Visibility {
            change_set_id: user_1_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_1.pk()),
    };

    let builder_result = ctx_builder.build(request_context).await;
    if let Err(e) = &builder_result {
        dbg!(e);
    }
    assert!(builder_result.is_ok());

    let request_context = RequestContext {
        tenancy: dal::Tenancy::new(*user_2_workspace.pk()),
        visibility: dal::Visibility {
            change_set_id: user_2_workspace.default_change_set_id(),
        },
        history_actor: HistoryActor::User(user_2.pk()),
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
    let mut change_set = ChangeSet::find(ctx, new_change_set.id)
        .await
        .expect("could not find change set")
        .expect("change set is some");

    change_set
        .request_change_set_approval(ctx)
        .await
        .expect("could not request approval");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update");
    let mut change_set = ChangeSet::find(ctx, new_change_set.id)
        .await
        .expect("could not find change set")
        .expect("change set is some");

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
    let mut change_set = ChangeSet::find(ctx, new_change_set.id)
        .await
        .expect("could not find change set")
        .expect("change set is some");
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
    let mut change_set = ChangeSet::find(ctx, new_change_set.id)
        .await
        .expect("could not find change set")
        .expect("change set is some");
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
    let mut change_set = ChangeSet::find(ctx, new_change_set.id)
        .await
        .expect("could not find change set")
        .expect("change set is some");

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
    let change_set = ChangeSet::find(ctx, new_change_set.id)
        .await
        .expect("could not find change set")
        .expect("change set is some");

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
