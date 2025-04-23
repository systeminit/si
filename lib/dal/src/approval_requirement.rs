//! This module provides functionality for creating, updating, deleting, getting and listing
//! approval requirements when applying a change set.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    // TODO(nick): restore this and clean up the mess.
    // missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::collections::{
    HashMap,
    HashSet,
};

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::Change,
};
use si_id::{
    ApprovalRequirementDefinitionId,
    EntityId,
    UserPk,
    ulid::Ulid,
};
use telemetry::prelude::*;
use thiserror::Error;

pub use crate::workspace_snapshot::traits::approval_requirement::{
    ApprovalRequirementApprover,
    ApprovalRequirementRule,
};
use crate::{
    DalContext,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventResult,
    WsPayload,
    layer_db_types::ApprovalRequirementDefinitionContentV1,
    workspace_snapshot::traits::approval_requirement::ApprovalRequirementExt,
};

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ApprovalRequirementError {
    #[error("Entity not found: {0}")]
    EntityNotFound(EntityId),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type Result<T> = std::result::Result<T, ApprovalRequirementError>;

#[derive(Debug)]
pub struct ApprovalRequirementExplicit {
    pub id: ApprovalRequirementDefinitionId,
    pub rule: ApprovalRequirementRule,
}

#[derive(Debug)]
pub enum ApprovalRequirement {
    Explicit(ApprovalRequirementExplicit),
    Virtual(ApprovalRequirementRule),
}

impl ApprovalRequirement {
    #[instrument(
        name = "approval_requirement.new_definition",
        level = "debug",
        skip_all
    )]
    pub async fn new_definition(
        ctx: &DalContext,
        entity_id: impl Into<Ulid>,
        minimum_approvers_count: usize,
        approvers: HashSet<ApprovalRequirementApprover>,
    ) -> Result<ApprovalRequirementDefinitionId> {
        ctx.workspace_snapshot()?
            .new_definition(ctx, entity_id.into(), minimum_approvers_count, approvers)
            .await
            .map_err(Into::into)
    }

    #[instrument(
        name = "approval_requirement.remove_definition",
        level = "debug",
        skip_all
    )]
    pub async fn remove_definition(
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
    ) -> Result<()> {
        ctx.workspace_snapshot()?
            .remove_definition(id)
            .await
            .map_err(Into::into)
    }

    #[instrument(
        name = "approval_requirement.add_individual_approver_for_definition",
        level = "debug",
        skip_all
    )]
    pub async fn add_individual_approver_for_definition(
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> Result<()> {
        ctx.workspace_snapshot()?
            .add_individual_approver_for_definition(ctx, id, user_id)
            .await
            .map_err(Into::into)
    }

    #[instrument(
        name = "approval_requirement.remove_individual_approver_for_definition",
        level = "debug",
        skip_all
    )]
    pub async fn remove_individual_approver_for_definition(
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> Result<()> {
        ctx.workspace_snapshot()?
            .remove_individual_approver_for_definition(ctx, id, user_id)
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "approval_requirement.list", level = "debug", skip_all)]
    pub async fn list(
        ctx: &DalContext,
        changes: &[Change],
    ) -> Result<(Vec<Self>, HashMap<EntityId, MerkleTreeHash>)> {
        ctx.workspace_snapshot()?
            .approval_requirements_for_changes(ctx, changes)
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequirementDefinition {
    pub id: ApprovalRequirementDefinitionId,
    pub required_count: usize,
    pub approvers: HashSet<ApprovalRequirementApprover>,
}

impl ApprovalRequirementDefinition {
    #[instrument(
        name = "approval_requirement_definition.list_for_entity_id",
        level = "debug",
        skip_all,
        fields(entity_id)
    )]
    pub async fn list_for_entity_id(
        ctx: &DalContext,
        entity_id: impl Into<Ulid>,
    ) -> Result<Vec<Self>> {
        let entity_id: EntityId = entity_id.into().into();
        if let Some(approval_requirement_definitions) = ctx
            .workspace_snapshot()?
            .approval_requirement_definitions_for_entity_id_opt(ctx, entity_id)
            .await?
        {
            return Ok(approval_requirement_definitions);
        }

        Err(ApprovalRequirementError::EntityNotFound(entity_id))
    }

    pub fn assemble(
        id: ApprovalRequirementDefinitionId,
        content: ApprovalRequirementDefinitionContentV1,
    ) -> Self {
        Self {
            id,
            required_count: content.minimum,
            approvers: content.approvers,
        }
    }
    pub async fn get_by_id(ctx: &DalContext, id: ApprovalRequirementDefinitionId) -> Result<Self> {
        ctx.workspace_snapshot()?
            .get_approval_requirement_definition_by_id(ctx, id)
            .await
            .map_err(Into::into)
    }

    pub async fn entity_id_for_approval_requirement_definition_id(
        ctx: &DalContext,
        id: ApprovalRequirementDefinitionId,
    ) -> Result<EntityId> {
        ctx.workspace_snapshot()?
            .entity_id_for_approval_requirement_definition_id(id)
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequirementDefinitionCreatedPayload {
    entity_id: EntityId,
    approvers: Option<Vec<UserPk>>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequirementDefinitionRemovedPayload {
    approval_requirement_definition_id: ApprovalRequirementDefinitionId,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndividualApproverPayload {
    approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    user_id: UserPk,
}

impl WsEvent {
    pub async fn requirement_created(
        ctx: &DalContext,
        entity_id: EntityId,
        approvers: Option<Vec<UserPk>>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ApprovalRequirementDefinitionCreated(
                ApprovalRequirementDefinitionCreatedPayload {
                    entity_id,
                    approvers,
                },
            ),
        )
        .await
    }

    pub async fn requirement_removed(
        ctx: &DalContext,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ApprovalRequirementDefinitionRemoved(
                ApprovalRequirementDefinitionRemovedPayload {
                    approval_requirement_definition_id,
                },
            ),
        )
        .await
    }

    pub async fn add_individual_approver_to_requirement(
        ctx: &DalContext,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ApprovalRequirementAddIndividualApprover(IndividualApproverPayload {
                approval_requirement_definition_id,
                user_id,
            }),
        )
        .await
    }

    pub async fn remove_individual_approver_from_requirement(
        ctx: &DalContext,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        user_id: UserPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ApprovalRequirementRemoveIndividualApprover(IndividualApproverPayload {
                approval_requirement_definition_id,
                user_id,
            }),
        )
        .await
    }
}
