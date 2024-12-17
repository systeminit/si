use std::collections::HashSet;

use dal::change_set::approval::{ChangeSetApproval, ChangeSetApprovalStatus};
use dal::{DalContext, Ulid};
use dal_test::color_eyre::eyre::OptionExt;
use dal_test::helpers::{
    create_component_for_default_schema_name_in_default_view, ChangeSetTestHelpers,
};
use dal_test::{test, Result};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &mut DalContext) -> Result<()> {
    create_component_for_default_schema_name_in_default_view(ctx, "fallout", "soken").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let status = ChangeSetApprovalStatus::Approved;
    let new_approval = ChangeSetApproval::new(ctx, status).await?;
    assert_eq!(
        status,                // expectd
        new_approval.status()  // actual
    );

    let mut approvals = ChangeSetApproval::list(ctx).await?;
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

#[test]
async fn status(ctx: &mut DalContext) -> Result<()> {
    create_component_for_default_schema_name_in_default_view(ctx, "fallout", "find the flame")
        .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let changes = ctx
        .workspace_snapshot()?
        .detect_changes_from_head(ctx)
        .await?;
    let seen: HashSet<Ulid> = HashSet::from_iter(changes.iter().map(|c| c.id));
    assert_eq!(
        changes.len(), // expected
        seen.len()     // actual
    );

    Ok(())
}
