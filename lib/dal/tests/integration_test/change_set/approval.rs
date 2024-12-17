use dal::change_set::approval::{ChangeSetApproval, ChangeSetApprovalStatus};
use dal::DalContext;
use dal_test::helpers::create_component_for_default_schema_name_in_default_view;
use dal_test::prelude::*;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new_and_list_latest(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "fallout", "soken").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let status = ChangeSetApprovalStatus::Approved;
    let raw_id: si_id::ulid::Ulid = component.id().into();
    let new_approval = ChangeSetApproval::new(ctx, status, vec![raw_id.into()]).await?;
    assert_eq!(
        status,                // expectd
        new_approval.status()  // actual
    );

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
