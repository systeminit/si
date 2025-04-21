use std::collections::HashSet;

use axum::{Json, extract::Path};
use dal::{
    ChangeSet, ChangeSetId, UserPk, WorkspacePk, WsEvent,
    approval_requirement::{ApprovalRequirement, ApprovalRequirementApprover},
    entity_kind::EntityKind,
};
use serde::Deserialize;
use si_events::audit_log::AuditLogKind;
use si_id::EntityId;

use crate::{
    extract::{HandlerContext, PosthogEventTracker},
    service::{force_change_set_response::ForceChangeSetResponse, v2::AccessBuilder},
};

use super::ApprovalRequirementDefinitionError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    entity_id: EntityId,
    // permission_lookups: Option<Vec<ApprovalRequirementPermissionLookup>>, // TODO(wendy) - this is not being used yet
    users: Option<Vec<UserPk>>,
}

pub async fn new(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<Request>,
) -> Result<ForceChangeSetResponse<()>, ApprovalRequirementDefinitionError> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut approvers = HashSet::new();

    if let Some(users) = request.users.to_owned() {
        approvers.extend(users.into_iter().map(ApprovalRequirementApprover::User));
    }

    // TODO(nick): add audit logs, posthog tracking and WsEvent(s).
    let approval_requirement_definition_id = ApprovalRequirement::new_definition(
        &ctx,
        request.entity_id,
        1, // TODO(nick): allow users to change the minimum approvers count
        approvers,
    )
    .await?;

    let entity_kind = EntityKind::get_entity_kind_for_id(&ctx, request.entity_id).await?;
    let entity_name = EntityKind::get_entity_name_for_id(&ctx, request.entity_id).await?;
    let title = format!(
        "{entity_kind} - {}",
        entity_name
            .to_owned()
            .unwrap_or_else(|| request.entity_id.to_string())
    );

    ctx.write_audit_log(
        AuditLogKind::CreateApprovalRequirementDefinition {
            individual_approvers: request.users.to_owned().unwrap_or(Vec::new()),
            approval_requirement_definition_id,
            entity_name: entity_name.to_owned(),
            entity_kind: entity_kind.to_string(),
            entity_id: request.entity_id,
        },
        title.to_owned(),
    )
    .await?;

    tracker.track(
        &ctx,
        "create_approval_requirement",
        serde_json::json!({
            "entity_kind": entity_kind.to_string(),
            "entity_name": title,
        }),
    );

    WsEvent::requirement_created(&ctx, request.entity_id, request.users)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
