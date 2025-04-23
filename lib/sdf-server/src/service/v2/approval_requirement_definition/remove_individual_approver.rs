use axum::extract::Path;
use dal::{
    ChangeSet,
    ChangeSetId,
    UserPk,
    WorkspacePk,
    WsEvent,
    approval_requirement::{
        ApprovalRequirement,
        ApprovalRequirementDefinition,
    },
    entity_kind::EntityKind,
};
use si_events::audit_log::AuditLogKind;
use si_id::ApprovalRequirementDefinitionId;

use super::ApprovalRequirementDefinitionError;
use crate::{
    extract::{
        HandlerContext,
        PosthogEventTracker,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
};

pub async fn remove_individual_approver(
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

    ApprovalRequirement::remove_individual_approver_for_definition(
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
        AuditLogKind::RemoveApprover {
            approval_requirement_definition_id,
            entity_name: entity_name.to_owned(),
            entity_id,
            entity_kind: entity_kind.to_string(),
            user_id,
        },
        title.to_owned(),
    )
    .await?;

    tracker.track(
        &ctx,
        "remove_individual_requirement",
        serde_json::json!({
            "entity_kind": entity_kind.to_string(),
            "entity_name": title,
        }),
    );

    WsEvent::remove_individual_approver_from_requirement(
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
