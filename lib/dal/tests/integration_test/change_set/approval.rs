use dal::change_set::approval::{ChangeSetApproval, ChangeSetApprovalStatus};
use dal::DalContext;
use dal_test::helpers::create_component_for_default_schema_name_in_default_view;
use dal_test::prelude::*;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new_and_list_latest(ctx: &mut DalContext) -> Result<()> {
    // Create a component and commit.
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "fallout", "soken").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Cache information we need.
    let status = ChangeSetApprovalStatus::Approved;
    let component_node_weight = ctx
        .workspace_snapshot()?
        .get_node_weight_by_id(component.id())
        .await?;
    let approving_ids_with_hashes = vec![(
        component_node_weight.id().into(),
        component_node_weight.merkle_tree_hash(),
    )];

    // Create an approval.
    let new_approval = ChangeSetApproval::new(ctx, status, approving_ids_with_hashes).await?;
    assert_eq!(
        status,                // expectd
        new_approval.status()  // actual
    );

    // List latest approvals.
    let mut approvals = ChangeSetApproval::list_latest(ctx).await?;
    let approval = approvals.pop().ok_or_eyre("unexpected empty approvals")?;
    assert!(approvals.is_empty());
    assert_eq!(
        new_approval.status(), // expected
        approval.status()      // actual
    );
    assert_eq!(
        new_approval.id(), // expected
        approval.id()      // actual
    );

    Ok(())
}
