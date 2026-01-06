//! This module provides the ability to insert and fetch [policy reports](PolicyReport).

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

use std::str::FromStr;

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgRow;
use si_id::{
    ChangeSetId,
    PolicyReportId,
    UserPk,
    WorkspacePk,
};
use thiserror::Error;

use crate::{
    SiDbContext,
    SiDbTransactions,
};

/// The default limit when listing [policy reports](PolicyReport).
pub(crate) const DEFAULT_LIST_LIMIT: i64 = 200;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PolicyReportError {
    #[error("could not parse policy report result: {0}")]
    ParsePolicyReportResult(#[source] strum::ParseError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] crate::Error),
    #[error("si db transactions error: {0}")]
    SiDbTransactions(#[from] crate::transactions::SiDbTransactionsError),
}

type Result<T> = std::result::Result<T, PolicyReportError>;

/// The report generated after evaluating a policy document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyReport {
    /// The unique identifier of the report.
    pub id: PolicyReportId,
    /// The workspace the policy report belongs to.
    pub workspace_id: WorkspacePk,
    /// The change set the policy report was generated for.
    pub change_set_id: ChangeSetId,
    /// The user that generated the policy report.
    pub user_id: Option<UserPk>,
    /// When the policy report was generated.
    pub created_at: DateTime<Utc>,
    /// The unique name of the policy report.
    pub name: String,
    /// The policy document that was used for evaluation.
    pub policy: String,
    /// The contents of the report.
    pub report: String,
    /// The shorthand result of the report.
    pub result: PolicyReportResult,
}

/// The result of a given [`PolicyReport`].
#[remain::sorted]
#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq, strum::EnumString, strum::Display,
)]
#[serde(rename_all = "PascalCase")]
pub enum PolicyReportResult {
    /// Indicates that the policy report result was a failure.
    Fail,
    /// Indicates that the policy report result was a success.
    Pass,
}

impl TryFrom<PgRow> for PolicyReport {
    type Error = PolicyReportError;

    fn try_from(row: PgRow) -> std::result::Result<Self, Self::Error> {
        let result_string: String = row.try_get("result")?;
        let result = PolicyReportResult::from_str(&result_string)
            .map_err(PolicyReportError::ParsePolicyReportResult)?;

        Ok(Self {
            id: row.try_get("id")?,
            workspace_id: row.try_get("workspace_id")?,
            change_set_id: row.try_get("change_set_id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            policy: row.try_get("policy")?,
            report: row.try_get("report")?,
            result,
            created_at: row.try_get("created_at")?,
        })
    }
}

impl PolicyReport {
    /// Inserts a new policy report entry with a passing [result](PolicyReportResult).
    pub async fn new_pass(
        ctx: &impl SiDbContext,
        name: String,
        policy: String,
        report: String,
    ) -> Result<Self> {
        Self::new_inner(ctx, name, policy, report, PolicyReportResult::Pass).await
    }

    /// Inserts a new policy report entry with a failing [result](PolicyReportResult).
    pub async fn new_fail(
        ctx: &impl SiDbContext,
        name: String,
        policy: String,
        report: String,
    ) -> Result<Self> {
        Self::new_inner(ctx, name, policy, report, PolicyReportResult::Fail).await
    }

    async fn new_inner(
        ctx: &impl SiDbContext,
        name: String,
        policy: String,
        report: String,
        result: PolicyReportResult,
    ) -> Result<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO policy_reports (
                    workspace_id,
                    change_set_id,
                    user_id,
                    name,
                    policy,
                    report,
                    result
                ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
                &[
                    &ctx.tenancy().workspace_pk()?,
                    &ctx.change_set_id(),
                    &ctx.history_actor().user_pk(),
                    &name,
                    &policy,
                    &report,
                    &result.to_string(),
                ],
            )
            .await?;

        Self::try_from(row)
    }

    /// List all [`PolicyReports`](PolicyReport) for the current workspace and change set. Limits
    /// the number of results with a [default](DEFAULT_LIST_LIMIT) limit.
    pub async fn list(ctx: &impl SiDbContext) -> Result<Vec<Self>> {
        Self::list_inner(ctx, DEFAULT_LIST_LIMIT).await
    }

    /// List all [`PolicyReports`](PolicyReport) for the current workspace and change set with a
    /// provided limit.
    pub async fn list_with_limit(ctx: &impl SiDbContext, limit: i64) -> Result<Vec<Self>> {
        Self::list_inner(ctx, limit).await
    }

    async fn list_inner(ctx: &impl SiDbContext, limit: i64) -> Result<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * FROM policy_reports WHERE workspace_id = $1 AND change_set_id = $2 ORDER BY created_at DESC LIMIT $3",
                &[&ctx.tenancy().workspace_pk()?, &ctx.change_set_id(), &limit],
            )
            .await?;

        let mut reports = Vec::with_capacity(rows.len());
        for row in rows {
            reports.push(Self::try_from(row)?);
        }

        Ok(reports)
    }
}
