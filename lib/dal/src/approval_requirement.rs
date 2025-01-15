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
    layer_db_types::{ApprovalRequirementContent, ApprovalRequirementContentV1},
    workspace_snapshot::{
        graph::detector::Change,
        node_weight::{traits::SiNodeWeight, NodeWeight, NodeWeightError},
        EntityKindExt,
    },
    DalContext, EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants, TransactionsError,
    WorkspaceSnapshotError,
};

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ApprovalRequirementError {
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type Result<T> = std::result::Result<T, ApprovalRequirementError>;

/// For a required approver, this is the permission lookup information needed to determine who can
/// satisfy the [requirement](ApprovalRequirement).
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequirementPermissionLookup {
    /// The object type in SpiceDB.
    pub object_type: String,
    /// The object ID in SpiceDB.
    pub object_id: String,
    /// The permission in SpiceDB.
    pub permission: String,
}

/// An approver within a [requirement](ApprovalRequirement).
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRequirementApprover {
    /// The approver is an individual user.
    User(UserPk),
    /// The approver can be multiple individuals, which can be found via a
    /// [permission lookup](ApprovalRequirementPermissionLookup).
    PermissionLookup(ApprovalRequirementPermissionLookup),
}

/// The requirement for a given [entity](EntityKind).
#[derive(Debug)]
pub struct ApprovalRequirement {
    #[allow(dead_code)]
    id: ApprovalRequirementId,
    entity_id: EntityId,
    entity_kind: EntityKind,
    minimum: usize,
    approvers: Vec<ApprovalRequirementApprover>,
}

impl ApprovalRequirement {
    /// Creates a new approval requirement for a given [entity](EntityKind).
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
            .add_edge(entity_id, EdgeWeight::new(EdgeWeightKind::Require), id)
            .await?;

        Ok(())
    }

    #[allow(missing_docs)]
    pub fn approvers(&self) -> &[ApprovalRequirementApprover] {
        &self.approvers
    }

    #[allow(missing_docs)]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    #[allow(missing_docs)]
    pub fn entity_kind(&self) -> EntityKind {
        self.entity_kind
    }

    #[allow(missing_docs)]
    pub fn minimum(&self) -> usize {
        self.minimum
    }

    /// Lists all approvals for a given set of changes.
    #[instrument(name = "approval_requirement.list", level = "debug", skip_all)]
    pub async fn list(ctx: &DalContext, changes: &[Change]) -> Result<Vec<Self>> {
        let snapshot = ctx.workspace_snapshot()?;

        let mut requirements = Vec::new();
        for change in changes {
            let entity_id: EntityId = change.id.into();

            for requirement_node_index in snapshot
                .outgoing_targets_for_edge_weight_kind(
                    entity_id,
                    EdgeWeightKindDiscriminants::Require,
                )
                .await?
            {
                let requirement_node_weight = snapshot
                    .get_node_weight(requirement_node_index)
                    .await?
                    .get_approval_requirement_node_weight()?;
                let hash = requirement_node_weight.content_hash();
                let approval_requirement_id: ApprovalRequirementId =
                    requirement_node_weight.id().into();

                // TODO(nick): collect all the hashes and perform one batch call instead.
                let content: ApprovalRequirementContent =
                    ctx.layer_db().cas().try_read_as(&hash).await?.ok_or(
                        WorkspaceSnapshotError::MissingContentFromStore(
                            approval_requirement_id.into(),
                        ),
                    )?;

                // NOTE(nick): if we had a v2, then there would be migration logic here.
                let ApprovalRequirementContent::V1(inner) = content;

                requirements.push(Self::assemble(
                    approval_requirement_id,
                    entity_id,
                    // TODO(nick): check if the entity extension trait at the graph level is still necessary.
                    snapshot.get_entity_kind_for_id(entity_id).await?,
                    inner,
                ));
            }
        }

        Ok(requirements)
    }

    fn assemble(
        id: ApprovalRequirementId,
        entity_id: EntityId,
        entity_kind: EntityKind,
        inner: ApprovalRequirementContentV1,
    ) -> Self {
        Self {
            id,
            entity_id,
            entity_kind,
            minimum: inner.minimum,
            approvers: inner.approvers,
        }
    }
}
