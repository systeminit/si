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

/// The maximum number of reports to return per group when fetching grouped reports.
pub const MAX_REPORTS_PER_GROUP: i64 = 10;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PolicyReportError {
    #[error("look up error, record not found")]
    LookupError,
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
    /// The change set name the policy report was generated for.
    pub change_set_name: String,
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
            change_set_name: row.try_get("change_set_name")?,
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

/// Represents a group of policy reports with the same name.
#[derive(Debug)]
pub struct PolicyReportGroup {
    /// The name of the policy.
    pub name: String,
    /// The latest reports for this policy (up to 10).
    pub results: Vec<PolicyReport>,
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

        let policy_id = row.try_get("id")?;
        let lookup_report = Self::fetch_single(ctx, policy_id).await?;
        if let Some(lookup_report) = lookup_report {
            Ok(lookup_report)
        } else {
            Err(PolicyReportError::LookupError)
        }
    }

    /// Fetches a single [`PolicyReport`](PolicyReport) for the current workspace based on ID in the url path.
    pub async fn fetch_single(
        ctx: &impl SiDbContext,
        id: PolicyReportId,
    ) -> Result<Option<PolicyReport>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT policy_reports.*, change_set_pointers.name as change_set_name FROM policy_reports INNER JOIN change_set_pointers ON change_set_pointers.id = policy_reports.change_set_id WHERE policy_reports.workspace_id = $1 AND policy_reports.id = $2",
                &[&ctx.tenancy().workspace_pk()?, &id],
            )
            .await?;
        let mut reports = Vec::with_capacity(rows.len());
        for row in rows {
            reports.push(Self::try_from(row)?);
        }
        let report = reports.into_iter().next();
        Ok(report)
    }

    /// Fetches a batch of [`PolicyReports`](PolicyReport) for the current workspace via pagination.
    /// Bundles the data into a [`PolicyReportBatch`].
    ///
    /// - If the caller does not provide a page size, the [`DEFAULT_PAGE_SIZE`] is used
    /// - If the caller does not provide a page number, the [`DEFAULT_PAGE_NUMBER`] is used
    /// - If the caller provides a page size that isn't greater than zero, the page size will be `1`
    /// - If the caller provides a page number that isn't greater than zero, the page number will be `1`
    /// - Only reports matching the provided name will be returned
    pub async fn fetch_batch(
        ctx: &impl SiDbContext,
        page_size: Option<u64>,
        page_number: Option<u64>,
        name: String,
    ) -> Result<PolicyReportBatch> {
        // Fallback to the default page size and page number, as needed. We also ensure that
        // the page number and size are at least "1".
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).max(1);
        let page_number = page_number.unwrap_or(DEFAULT_PAGE_NUMBER).max(1);

        // Collect the paginated reports. We need an offset if we are beyond the first page.
        let offset = page_number.saturating_sub(1).saturating_mul(page_size);

        let rows = ctx.txns().await?.pg()
            .query(
                "SELECT policy_reports.*, change_set_pointers.name as change_set_name FROM policy_reports INNER JOIN change_set_pointers ON change_set_pointers.id = policy_reports.change_set_id WHERE policy_reports.workspace_id = $1 AND policy_reports.name = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                &[
                    &ctx.tenancy().workspace_pk()?,
                    &name,
                    &(page_size as i64),
                    &(offset as i64),
                ],
            )
            .await?;

        let mut reports = Vec::with_capacity(rows.len());
        for row in rows {
            reports.push(Self::try_from(row)?);
        }

        // Determine the total number of reports available within the workspace.
        // Then, derive the total number of pages from that.
        let total_report_count = Self::total_count(ctx, &name).await?;
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

    /// Fetches policy reports grouped by name, with the latest reports for each name.
    /// Returns a vector of [`PolicyReportGroup`] objects, each containing a policy name and its latest reports.
    /// The number of reports per group is limited by [`MAX_REPORTS_PER_GROUP`].
    pub async fn fetch_grouped_by_name(ctx: &impl SiDbContext) -> Result<Vec<PolicyReportGroup>> {
        // Use a window function to get the latest N reports per policy name in a single query
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT *, change_set_pointers.name as change_set_name FROM (
                    SELECT *, ROW_NUMBER() OVER (PARTITION BY name ORDER BY created_at DESC) as rn
                    FROM policy_reports
                    WHERE workspace_id = $1
                ) ranked
                INNER JOIN change_set_pointers ON change_set_pointers.id = ranked.change_set_id
                WHERE rn <= $2
                ORDER BY ranked.name, ranked.created_at DESC",
                &[&ctx.tenancy().workspace_pk()?, &MAX_REPORTS_PER_GROUP],
            )
            .await?;

        // Group the reports by name
        let mut groups_map: std::collections::HashMap<String, Vec<PolicyReport>> =
            std::collections::HashMap::new();

        for row in rows {
            let report = Self::try_from(row)?;
            groups_map
                .entry(report.name.clone())
                .or_default()
                .push(report);
        }

        // Convert the HashMap into a Vec of PolicyReportGroup, sorted by name
        let mut groups: Vec<PolicyReportGroup> = groups_map
            .into_iter()
            .map(|(name, results)| PolicyReportGroup { name, results })
            .collect();

        groups.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(groups)
    }

    async fn total_count(ctx: &impl SiDbContext, name: &str) -> Result<u64> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT COUNT(*) FROM policy_reports WHERE workspace_id = $1 AND name = $2",
                &[&ctx.tenancy().workspace_pk()?, &name],
            )
            .await?;

        // I want to turn ToSql into a flea, a harmless little flea, and then I'll put that flea in
        // a box, and then I'll put that box inside of another box, and then I'll mail that box to
        // myself. And when it arrives, I'LL SMASH IT WITH A HAMMER. Pain.
        let bigint: i64 = row.try_get("count")?;
        Ok(bigint as u64)
    }
}
