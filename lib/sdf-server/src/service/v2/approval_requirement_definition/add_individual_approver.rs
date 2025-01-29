use axum::extract::Path;
use dal::{
    approval_requirement::ApprovalRequirement, ChangeSet, ChangeSetId, UserPk, WorkspacePk, WsEvent,
};
use si_id::ApprovalRequirementDefinitionId;

use crate::{
    extract::HandlerContext, service::force_change_set_response::ForceChangeSetResponse,
    service::v2::AccessBuilder,
};

use super::ApprovalRequirementDefinitionError;

pub async fn add_individual_approver(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, approval_requirement_definition_id, user_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ApprovalRequirementDefinitionId,
        UserPk,
    )>,
) -> Result<ForceChangeSetResponse<()>, ApprovalRequirementDefinitionError> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // TODO(nick): add audit logs, posthog tracking and WsEvent(s).
    ApprovalRequirement::add_individual_approver_for_definition(
        &ctx,
        approval_requirement_definition_id,
        user_id,
    )
    .await?;

    WsEvent::add_individual_approver_to_requirement(
        &ctx,
        approval_requirement_definition_id,
        user_id,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
