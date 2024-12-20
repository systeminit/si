//! Provides the ability to approve change sets and calculate their approval status.

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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgRow};
use si_events::ChangesChecksum;
use si_id::{ChangeSetApprovalId, ChangeSetId, UserPk};
use telemetry::prelude::*;
use thiserror::Error;

pub use si_events::ChangeSetApprovalStatus;

use crate::{DalContext, HistoryActor, TransactionsError, WorkspaceSnapshotError};

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ChangeSetApprovalError {
    #[error("invalid user for creating a change set approval")]
    InvalidUserForCreation,
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("strum parse error: {0}")]
    StrumParse(#[from] strum::ParseError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type Result<T> = std::result::Result<T, ChangeSetApprovalError>;

/// An individual approval for applying a [`ChangeSet`](crate::ChangeSet).
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeSetApproval {
    id: ChangeSetApprovalId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    change_set_id: ChangeSetId,
    status: ChangeSetApprovalStatus,
    user_id: UserPk,
    checksum: String,
}

impl TryFrom<PgRow> for ChangeSetApproval {
    type Error = ChangeSetApprovalError;

    fn try_from(value: PgRow) -> std::result::Result<Self, Self::Error> {
        let status_string: String = value.try_get("status")?;
        let status = ChangeSetApprovalStatus::try_from(status_string.as_str())?;
        Ok(Self {
            id: value.try_get("id")?,
            created_at: value.try_get("created_at")?,
            updated_at: value.try_get("updated_at")?,
            change_set_id: value.try_get("change_set_id")?,
            status,
            user_id: value.try_get("user_id")?,
            checksum: value.try_get("checksum")?,
        })
    }
}

impl ChangeSetApproval {
    /// Creates a new approval.
    #[instrument(name = "change_set.approval.new", level = "info", skip_all)]
    pub async fn new(ctx: &DalContext, status: ChangeSetApprovalStatus) -> Result<Self> {
        let change_set_id = ctx.change_set_id();
        let user_id = match ctx.history_actor() {
            HistoryActor::User(user_id) => user_id,
            HistoryActor::SystemInit => return Err(ChangeSetApprovalError::InvalidUserForCreation),
        };
        let checksum = Self::calculate_checksum(ctx).await?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO change_set_approvals (change_set_id, status, user_id, checksum) VALUES ($1, $2, $3, $4) RETURNING *",
                &[&change_set_id, &status.to_string(), &user_id, &checksum.to_string()]
            )
            .await?;
        Self::try_from(row)
    }

    /// Returns the ID of the approval.
    pub fn id(&self) -> ChangeSetApprovalId {
        self.id
    }

    /// Returns the status of the approval.
    pub fn status(&self) -> ChangeSetApprovalStatus {
        self.status
    }

    /// Returns the ID of the approver.
    pub fn user_id(&self) -> UserPk {
        self.user_id
    }

    /// Returns the checksum based on the changes compared to HEAD when the approval was performed.
    pub fn checksum(&self) -> &str {
        self.checksum.as_str()
    }

    /// Lists all approvals in the [`ChangeSet`](crate::ChangeSet).
    #[instrument(name = "change_set.approval.list", level = "info", skip_all)]
    pub async fn list(ctx: &DalContext) -> Result<Vec<Self>> {
        let change_set_id = ctx.change_set_id();
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_approvals WHERE change_set_id = $1 ORDER BY id ASC",
                &[&change_set_id],
            )
            .await?;
        let mut approvals = Vec::with_capacity(rows.len());
        for row in rows {
            approvals.push(Self::try_from(row)?);
        }
        Ok(approvals)
    }

    /// Calculates a checksum of all detected changes between the current [`ChangeSet`] and HEAD.
    pub async fn calculate_checksum(ctx: &DalContext) -> Result<ChangesChecksum> {
        let mut changes = ctx
            .workspace_snapshot()?
            .detect_changes_from_head(ctx)
            .await?;
        changes.sort_by_key(|c| c.id);
        let mut hasher = ChangesChecksum::hasher();
        for change in changes {
            hasher.update(change.merkle_tree_hash.as_bytes());
        }
        Ok(hasher.finalize())
    }
}
