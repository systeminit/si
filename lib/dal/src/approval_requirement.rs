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

use si_id::{ulid::Ulid, ApprovalRequirementDefinitionId};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    workspace_snapshot::{
        graph::detector::Change, traits::approval_requirement::ApprovalRequirementExt,
    },
    DalContext, WorkspaceSnapshotError,
};

pub use crate::workspace_snapshot::traits::approval_requirement::{
    ApprovalRequirementApprover, ApprovalRequirementRule,
};

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ApprovalRequirementError {
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
        approvers: Vec<ApprovalRequirementApprover>,
    ) -> Result<ApprovalRequirementDefinitionId> {
        ctx.workspace_snapshot()?
            .new_approval_requirement_definition(
                ctx,
                entity_id.into(),
                minimum_approvers_count,
                approvers,
            )
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "approval_requirement.list", level = "debug", skip_all)]
    pub async fn list(ctx: &DalContext, changes: &[Change]) -> Result<Vec<Self>> {
        ctx.workspace_snapshot()?
            .approval_requirements_for_changes(ctx, changes)
            .await
            .map_err(Into::into)
    }
}
