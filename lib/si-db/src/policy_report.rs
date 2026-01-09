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

/// The default page number when fetching a batch of [policy reports](PolicyReport).
pub const DEFAULT_PAGE_NUMBER: u64 = 1;

/// The default page size when fetching a batch of [policy reports](PolicyReport).
pub const DEFAULT_PAGE_SIZE: u64 = 200;

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

/// Contains a batch of [policy reports](PolicyReport) for a workspace and change set with relevant
/// metadata.
#[derive(Debug)]
pub struct PolicyReportBatch {
    /// A list of reports for a workspace and change set.
    pub reports: Vec<PolicyReport>,
    /// The page size used to fetch the list of reports.
    pub page_size: u64,
    /// The page number used to fetch the list of reports.
    pub page_number: u64,
    /// The total number of pages given the page size and total number of reports for the workspace
    /// and change set.
    pub total_page_count: u64,
    /// The total number of reports for the workspace and change set.
    pub total_report_count: u64,
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
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING *",
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

    /// Fetches a single [`PolicyReport`](PolicyReport) for the current workspace and change set based on ID in the url path.
    pub async fn fetch_single(
        ctx: &impl SiDbContext,
        id: PolicyReportId,
    ) -> Result<Option<PolicyReport>> {
        let rows =
            ctx.txns().await?.pg()
                .query(
                    "SELECT * FROM policy_reports WHERE workspace_id = $1 AND change_set_id = $2 and id = $3",
                    &[
                        &ctx.tenancy().workspace_pk()?,
                        &ctx.change_set_id(),
                        &id,
                    ],
                )
                .await?;
        let mut reports = Vec::with_capacity(rows.len());
        for row in rows {
            reports.push(Self::try_from(row)?);
        }
        let report = reports.into_iter().next();
        Ok(report)
    }

    /// Fetches a batch of [`PolicyReports`](PolicyReport) for the current workspace and change set via pagination.
    /// Bundles the data into a [`PolicyReportBatch`].
    ///
    /// - If the caller does not provide a page size, the [`DEFAULT_PAGE_SIZE`] is used
    /// - If the caller does not provide a page number, the [`DEFAULT_PAGE_NUMBER`] is used
    /// - If the caller provides a page size that isn't greater than zero, the page size will be `1`
    /// - If the caller provides a page number that isn't greater than zero, the page number will be `1`
    pub async fn fetch_batch(
        ctx: &impl SiDbContext,
        page_size: Option<u64>,
        page_number: Option<u64>,
    ) -> Result<PolicyReportBatch> {
        // Fallback to the default page size and page number, as needed. We also ensure that the
        // the page number and size are at least "1".
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).max(1);
        let page_number = page_number.unwrap_or(DEFAULT_PAGE_NUMBER).max(1);

        // Collect the paginated reports. We need an offset if we are beyond the first page.
        let rows = if page_number == 1 {
            ctx.txns().await?.pg()
                .query(
                    "SELECT * FROM policy_reports WHERE workspace_id = $1 AND change_set_id = $2 ORDER BY created_at DESC LIMIT $3",
                    &[
                        &ctx.tenancy().workspace_pk()?,
                        &ctx.change_set_id(),
                        &(page_size as i64), // required to make the query happy, but I want to strangle it
                    ],
                )
                .await?
        } else {
            let offset = page_number.saturating_sub(1).saturating_mul(page_size);
            ctx.txns().await?.pg()
                .query(
                    "SELECT * FROM policy_reports WHERE workspace_id = $1 AND change_set_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                    &[
                        &ctx.tenancy().workspace_pk()?,
                        &ctx.change_set_id(),
                        &(page_size as i64), // required to make the query happy, but I want to strangle it
                        &(offset as i64) // required to make the query happy, but I want to strangle it
                    ],
                )
                .await?
        };
        let mut reports = Vec::with_capacity(rows.len());
        for row in rows {
            reports.push(Self::try_from(row)?);
        }

        // Determine the total number of reports available within the workspace and change set.
        // Then, derive the total number of pages from that.
        let total_report_count = Self::total_count(ctx).await?;
        let total_page_count = if total_report_count == 0 {
            0
        } else {
            // Ceiling division makes it so that there's always another page if at least one entry
            // is on it. I LOVE THE NUMBERS.
            total_report_count.div_ceil(page_size)
        };

        Ok(PolicyReportBatch {
            reports,
            page_size,
            page_number,
            total_page_count,
            total_report_count,
        })
    }

    async fn total_count(ctx: &impl SiDbContext) -> Result<u64> {
        let row = ctx.txns().await?.pg()
            .query_one(
                "SELECT COUNT(*) FROM policy_reports WHERE workspace_id = $1 AND change_set_id = $2",
                &[
                    &ctx.tenancy().workspace_pk()?,
                    &ctx.change_set_id()
                ],
            )
            .await?;

        // I want to turn ToSql into a flea, a harmless little flea, and then I'll put that flea in
        // a box, and then I'll put that box inside of another box, and then I'll mail that box to
        // myself. And when it arrives, I'LL SMASH IT WITH A HAMMER. Pain.
        let bigint: i64 = row.try_get("count")?;
        Ok(bigint as u64)
    }
}
