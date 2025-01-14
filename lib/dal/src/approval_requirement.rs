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
    missing_docs,
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

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use si_events::workspace_snapshot::EntityKind;
use si_id::{ulid::Ulid, ApprovalRequirementId, EntityId, UserPk};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    implement_add_edge_to,
    layer_db_types::{ApprovalRequirementContent, ApprovalRequirementContentV1},
    workspace_snapshot::{graph::detector::Change, node_weight::NodeWeight, EntityKindExt},
    DalContext, EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants, TransactionsError,
    WorkspaceSnapshotError,
};

#[derive(Debug, Error)]
pub enum ApprovalRequirementError {
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type Result<T> = std::result::Result<T, ApprovalRequirementError>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequirementPermissionLookup {
    pub object_type: String,
    pub object_id: String,
    pub permission: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRequirementApprover {
    User(UserPk),
    PermissionLookup(ApprovalRequirementPermissionLookup),
}

#[derive(Debug)]
pub struct ApprovalRequirement {
    id: ApprovalRequirementId,
    entity_id: EntityId,
    entity_kind: EntityKind,
    number: usize,
    lookup_groups: Vec<ApprovalRequirementApprover>,
}

impl ApprovalRequirement {
    #[instrument(name = "approval_requirement.new", level = "debug", skip_all)]
    pub async fn new(ctx: &DalContext, entity_id: impl Into<Ulid>) -> Result<()> {
        let workspace_id = ctx.workspace_pk()?;
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // TODO(nick): remove hard-coded contents.
        let content = ApprovalRequirementContentV1 {
            minimum: 1,
            approvers: vec![ApprovalRequirementApprover::PermissionLookup(
                ApprovalRequirementPermissionLookup {
                    object_type: "workspace".to_string(),
                    object_id: workspace_id.to_string(),
                    permission: "approve".to_string(),
                },
            )],
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(ApprovalRequirementContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;
        let node_weight = NodeWeight::new_approval_requirement(id, lineage_id, hash);
        workspace_snapshot.add_or_replace_node(node_weight).await?;

        ctx.workspace_snapshot()?
            .add_edge(
                entity_id,
                EdgeWeight::new(EdgeWeightKind::Use { is_default: false }),
                id,
            )
            .await?;

        Ok(())
    }

    #[instrument(name = "approval_requirement.list", level = "debug", skip_all)]
    pub async fn list(ctx: &DalContext, changes: &[Change]) -> Result<Vec<Self>> {
        let workspace_id = ctx.workspace_pk()?;

        let mut requirements = Vec::new();
        for change in changes {
            let entity_id: EntityId = change.id.into();

            // TODO(nick,jacob): handle more than schema variants.
            if let EntityKind::SchemaVariant = ctx
                .workspace_snapshot()?
                .get_entity_kind_for_id(entity_id)
                .await?
            {
                requirements.push(Self {
                    // TODO(nick): don't make this BS!
                    id: ApprovalRequirementId::new(),
                    // TODO(nick,jacob): handle more than schema variants.
                    entity_kind: EntityKind::SchemaVariant,
                    entity_id,
                    // TODO(nick,jacob): remove hardcoded number requirement.
                    number: 1,
                    // TODO(nick,jacob): replace hardcoded relations.
                    lookup_groups: vec![ApprovalRequirementApprover::PermissionLookup(
                        ApprovalRequirementPermissionLookup {
                            object_type: "workspace".to_string(),
                            object_id: workspace_id.to_string(),
                            permission: "approve".to_string(),
                        },
                    )],
                });
            }
        }
        Ok(requirements)
    }
}
