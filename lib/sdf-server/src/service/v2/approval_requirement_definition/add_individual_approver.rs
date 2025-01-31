use axum::extract::Path;
use dal::{
    approval_requirement::{ApprovalRequirement, ApprovalRequirementDefinition},
    entity_kind::EntityKind,
    ChangeSet, ChangeSetId, UserPk, WorkspacePk, WsEvent,
};
use si_events::audit_log::AuditLogKind;
use si_id::ApprovalRequirementDefinitionId;

use crate::{
    extract::{HandlerContext, PosthogEventTracker},
    service::{force_change_set_response::ForceChangeSetResponse, v2::AccessBuilder},
};

use super::ApprovalRequirementDefinitionError;

pub async fn add_individual_approver(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
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
    let entity_id =
        ApprovalRequirementDefinition::entity_id_for_approval_requirement_definition_id(
            &ctx,
            approval_requirement_definition_id,
        )
        .await?;

    let entity_kind = EntityKind::get_entity_kind_for_id(&ctx, entity_id).await?;
    let entity_name = EntityKind::get_entity_name_for_id(&ctx, entity_id).await?;
    let title = format!(
        "{entity_kind} - {}",
        entity_name
            .to_owned()
            .unwrap_or_else(|| entity_id.to_string())
    );

    ctx.write_audit_log(
        AuditLogKind::AddApprover {
            approval_requirement_definition_id,
            entity_name: entity_name.to_owned(),
            user_id,
            entity_kind: entity_kind.to_string(),
            entity_id,
        },
        title.to_owned(),
    )
    .await?;

    tracker.track(
        &ctx,
        "add_individual_approver",
        serde_json::json!({
            "entity_kind": entity_kind.to_string(),
            "entity_name": title,
        }),
    );

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
