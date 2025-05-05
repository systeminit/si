use std::collections::{
    HashMap,
    HashSet,
};

use dal::{
    DalContext,
    action::Action,
    approval_requirement::{
        ApprovalRequirement,
        ApprovalRequirementApprover,
    },
    change_set::approval::ChangeSetApproval,
    diagram::view::View,
};
use dal_test::{
    Result,
    eyre,
    helpers::create_component_for_default_schema_name,
    prelude::ChangeSetTestHelpers,
    sdf_test,
};
use indoc::indoc;
use pretty_assertions_sorted::assert_eq;
use sdf_core::{
    dal_wrapper,
    dal_wrapper::DalWrapperError,
};
use sdf_test::helpers::SdfTestHelpers;
use si_data_spicedb::SpiceDbClient;
use si_db::HistoryActor;
use si_events::{
    ChangeSetApprovalStatus,
    workspace_snapshot::EntityKind,
};
use si_id::ViewId;

// FIXME(nick,jacob): this must happen in the "sdf_test"'s equivalent to global setup, but not in
// dal tests. This also should _really_ reflect the "schema.zed" file that production uses.
async fn write_schema(client: &mut SpiceDbClient) -> Result<()> {
    let schema = indoc! {"
        definition user {}

        definition workspace {
          relation approver: user
          relation owner: user
          permission approve = approver+owner
          permission manage = owner
        }
    "};
    client.write_schema(schema).await?;
    Ok(())
}

// NOTE(nick): this is an integration test and not a service test, but given that "sdf_test" is in
// a weird, unused place at the time of writing, this test will live here.
#[sdf_test]
async fn protected_apply(ctx: &mut DalContext, spicedb_client: SpiceDbClient) -> Result<()> {
    let mut spicedb_client = spicedb_client;

    // FIXME(nick,jacob): see the comment attached to this function.
    write_schema(&mut spicedb_client).await?;

    // Cache the IDs we need.
    let user_id = match ctx.history_actor() {
        HistoryActor::SystemInit => return Err(eyre!("invalid user")),
        HistoryActor::User(user_id) => *user_id,
    };

    // Create a view with a requirement and then commit.
    let todd_view = View::new(ctx, "toddhoward").await?;
    let todd_view_id = todd_view.id();
    ApprovalRequirement::new_definition(
        ctx,
        todd_view_id,
        1,
        HashSet::from([ApprovalRequirementApprover::User(user_id)]),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Scenario 1: apply to HEAD and create a new change set.
    {
        ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
        ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

        let (frontend_latest_approvals, frontend_requirements) =
            dal_wrapper::change_set::status(ctx, &mut spicedb_client).await?;

        assert!(frontend_latest_approvals.is_empty());
        assert!(frontend_requirements.is_empty());
    }

    // Scenario 2: create a component in our new view.
    {
        let new_component = create_component_for_default_schema_name(
            ctx,
            "starfield",
            "shattered space",
            todd_view_id,
        )
        .await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
        let queued_actions = Action::find_for_component_id(ctx, new_component.id()).await?;
        assert_eq!(1, queued_actions.len());

        let (frontend_latest_approvals, mut frontend_requirements) =
            dal_wrapper::change_set::status(ctx, &mut spicedb_client).await?;
        frontend_requirements.sort_by_key(|r| r.entity_id);

        assert!(frontend_latest_approvals.is_empty());
        assert_eq!(
            vec![si_frontend_types::ChangeSetApprovalRequirement {
                entity_id: todd_view_id.into_inner().into(),
                entity_kind: EntityKind::View,
                required_count: 1,
                is_satisfied: false,
                applicable_approval_ids: Vec::new(),
                approver_groups: HashMap::new(),
                approver_individuals: vec![user_id],
            },], // expected
            frontend_requirements // actual
        );
    }

    // Scenario 3: try to perform protected apply. This should fail in an expected manner.
    {
        match SdfTestHelpers::protected_apply_change_set_to_base(ctx, &mut spicedb_client).await {
            Err(report) => match report.downcast_ref::<DalWrapperError>() {
                Some(DalWrapperError::ApplyWithUnsatisfiedRequirements(
                    unsatisfied_requirements,
                )) => assert_eq!(
                    vec![(todd_view_id.into_inner().into(), EntityKind::View)], // expected
                    unsatisfied_requirements.to_owned()                         // actual
                ),
                Some(err) => return Err(eyre!("unexpected error: {err}")),
                None => return Err(eyre!("unexpected report: {report:?}")),
            },
            other => return Err(eyre!("unexpected result: {other:?}")),
        }
    }

    // Scenario 4: approve the changes.
    {
        let approving_ids_with_hashes =
            dal_wrapper::change_set::new_approval_approving_ids_with_hashes(
                ctx,
                &mut spicedb_client,
            )
            .await?;
        let approval = ChangeSetApproval::new(
            ctx,
            ChangeSetApprovalStatus::Approved,
            approving_ids_with_hashes,
        )
        .await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let (frontend_latest_approvals, frontend_requirements) =
            dal_wrapper::change_set::status(ctx, &mut spicedb_client).await?;

        assert_eq!(
            vec![si_frontend_types::ChangeSetApproval {
                id: approval.id(),
                user_id,
                status: ChangeSetApprovalStatus::Approved,
                is_valid: true,
            }], // expected
            frontend_latest_approvals // actual
        );
        assert_eq!(
            vec![si_frontend_types::ChangeSetApprovalRequirement {
                entity_id: todd_view_id.into_inner().into(),
                entity_kind: EntityKind::View,
                required_count: 1,
                is_satisfied: true,
                applicable_approval_ids: vec![approval.id()],
                approver_groups: HashMap::new(),
                approver_individuals: vec![user_id]
            }], // expected
            frontend_requirements // actual
        );
    }

    // Scenario 5: apply the changes used the protected flow and observe that it works.
    {
        SdfTestHelpers::protected_apply_change_set_to_base(ctx, &mut spicedb_client).await?;
        ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;
        ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

        let default_view_id = View::get_id_for_default(ctx).await?;
        let mut view_ids: Vec<ViewId> = View::list(ctx).await?.iter().map(|v| v.id()).collect();
        view_ids.sort();

        assert_eq!(
            vec![default_view_id, todd_view_id], // expected
            view_ids                             // actual
        );

        let (frontend_latest_approvals, frontend_requirements) =
            dal_wrapper::change_set::status(ctx, &mut spicedb_client).await?;

        assert!(frontend_latest_approvals.is_empty());
        assert!(frontend_requirements.is_empty());
    }

    Ok(())
}
