use anyhow::Result;
use axum::extract::Path;
use dal::{
    approval_requirement::{
        ApprovalRequirement, ApprovalRequirementApprover, ApprovalRequirementDefinition,
    },
    entity_kind::EntityKind,
    ChangeSet, ChangeSetId, UserPk, WorkspacePk, WsEvent,
};
use si_events::audit_log::AuditLogKind;
use si_id::ApprovalRequirementDefinitionId;

use crate::{
    extract::{HandlerContext, PosthogEventTracker},
    service::{force_change_set_response::ForceChangeSetResponse, v2::AccessBuilder},
};

pub async fn remove(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, change_set_id, approval_requirement_definition_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ApprovalRequirementDefinitionId,
    )>,
) -> Result<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    // cache things before they're removed
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

    let current_approval_definition =
        ApprovalRequirementDefinition::get_by_id_or_error(&ctx, approval_requirement_definition_id)
            .await?;
    let individual_approvers: Vec<UserPk> = current_approval_definition
        .approvers
        .iter()
        .filter_map(|approver| {
            match approver {
                // TODO(brit/nick but mostly nick but actually brit): when we add groups, this will need to change
                ApprovalRequirementApprover::PermissionLookup(
                    _approval_requirement_permission_lookup,
                ) => None,
                ApprovalRequirementApprover::User(user_pk) => Some(*user_pk),
            }
        })
        .collect();

    ApprovalRequirement::remove_definition(&ctx, approval_requirement_definition_id).await?;

    ctx.write_audit_log(
        AuditLogKind::DeleteApprovalRequirementDefinition {
            approval_requirement_definition_id,
            entity_name: entity_name.to_owned(),
            entity_id,
            individual_approvers,
            entity_kind: entity_kind.to_string(),
        },
        title.to_owned(),
    )
    .await?;

    tracker.track(
        &ctx,
        "remove_approval_requirement_definition",
        serde_json::json!({
            "entity_kind": entity_kind.to_string(),
            "entity_name": title
        }),
    );

    WsEvent::requirement_removed(&ctx, approval_requirement_definition_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
