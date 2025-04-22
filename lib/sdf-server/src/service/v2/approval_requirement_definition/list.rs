use std::{collections::HashMap, str::FromStr};

use axum::{
    Json,
    extract::{Path, State},
};
use dal::{
    ChangeSetId, UserPk, WorkspacePk,
    approval_requirement::{
        ApprovalRequirementApprover,
        ApprovalRequirementDefinition as DalApprovalRequirementDefinition,
    },
    workspace_snapshot::EntityKindExt,
};
use si_frontend_types::ApprovalRequirementDefinition;
use si_id::EntityId;

use crate::{AppState, extract::HandlerContext, service::v2::AccessBuilder};

use super::ApprovalRequirementDefinitionError;

pub async fn list_for_entity(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, entity_id)): Path<(WorkspacePk, ChangeSetId, EntityId)>,
    State(mut state): State<AppState>,
) -> Result<Json<Vec<ApprovalRequirementDefinition>>, ApprovalRequirementDefinitionError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let entity_kind = ctx
        .workspace_snapshot()?
        .get_entity_kind_for_id(entity_id)
        .await?;

    let spicedb_client = state
        .spicedb_client()
        .ok_or(ApprovalRequirementDefinitionError::SpiceDbClientNotFound)?;

    let mut results = Vec::new();
    for definition in DalApprovalRequirementDefinition::list_for_entity_id(&ctx, entity_id).await? {
        let mut approver_groups = HashMap::new();
        let mut approver_individuals = Vec::new();
        for approver in definition.approvers {
            let permission_lookup = match approver {
                ApprovalRequirementApprover::PermissionLookup(
                    approval_requirement_permission_lookup,
                ) => approval_requirement_permission_lookup,
                ApprovalRequirementApprover::User(user_pk) => {
                    approver_individuals.push(user_pk);
                    continue;
                }
            };
            let permission_lookup_key = format!(
                "{}#{}#{}",
                permission_lookup.object_type,
                permission_lookup.object_id,
                permission_lookup.permission,
            );
            let raw_member_ids = spicedb_client
                .lookup_subjects(
                    permission_lookup.object_type.to_owned(),
                    permission_lookup.object_id.to_owned(),
                    permission_lookup.permission.to_owned(),
                    "user".to_owned(),
                )
                .await?;
            let mut member_ids = Vec::with_capacity(raw_member_ids.len());
            for raw_member_id in raw_member_ids {
                member_ids.push(UserPk::from_str(raw_member_id.as_str())?);
            }
            member_ids.sort();
            approver_groups.insert(permission_lookup_key, member_ids);
        }
        results.push(ApprovalRequirementDefinition {
            id: definition.id,
            entity_id,
            entity_kind,
            required_count: definition.required_count,
            approver_groups,
            approver_individuals,
        });
    }

    Ok(Json(results))
}
