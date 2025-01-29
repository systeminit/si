use std::collections::HashSet;

use axum::{extract::Path, Json};
use dal::{
    approval_requirement::{ApprovalRequirement, ApprovalRequirementApprover},
    // workspace_snapshot::graph::traits::approval_requirement::ApprovalRequirementPermissionLookup,
    ChangeSet,
    ChangeSetId,
    UserPk,
    WorkspacePk,
    WsEvent,
};
use serde::Deserialize;
use si_id::EntityId;

use crate::{
    extract::HandlerContext, service::force_change_set_response::ForceChangeSetResponse,
    service::v2::AccessBuilder,
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
    ApprovalRequirement::new_definition(
        &ctx,
        request.entity_id,
        1, // TODO(nick): allow users to change the minimum approvers count
        approvers,
    )
    .await?;

    WsEvent::requirement_created(&ctx, request.entity_id, request.users)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
