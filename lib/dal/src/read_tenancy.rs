use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    BillingAccountId, OrganizationId, StandardModelError, Visibility, WorkspaceError, WorkspaceId,
};

const GET_WORKSPACE: &str = include_str!("queries/read_tenancy/get_workspace.sql");
const GET_ORGANIZATION: &str = include_str!("queries/read_tenancy/get_organization.sql");

#[derive(Error, Debug)]
pub enum ReadTenancyError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace not found error: {0}")]
    WorkspaceNotFound(WorkspaceId),
    #[error("organization not found for workspace error: {0}")]
    OrganizationNotFoundForWorkspace(WorkspaceId),
    #[error("billing account not found for organization error: {0}")]
    BillingAccountNotFoundForOrganization(OrganizationId),
}

pub type ReadTenancyResult<T> = Result<T, ReadTenancyError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ReadTenancy {
    #[serde(rename = "tenancy_universal")]
    universal: bool,
    #[serde(rename = "tenancy_billing_account_ids")]
    billing_account_ids: Vec<BillingAccountId>,
    #[serde(rename = "tenancy_organization_ids")]
    organization_ids: Vec<OrganizationId>,
    #[serde(rename = "tenancy_workspace_ids")]
    workspace_ids: Vec<WorkspaceId>,
}

impl ReadTenancy {
    pub fn billing_accounts(&self) -> &[BillingAccountId] {
        &self.billing_account_ids
    }

    pub fn new_universal() -> Self {
        Self {
            universal: true,
            billing_account_ids: Vec::new(),
            organization_ids: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_billing_account(billing_account_ids: Vec<BillingAccountId>) -> Self {
        Self {
            universal: true,
            billing_account_ids,
            organization_ids: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }

    pub async fn new_organization(
        txn: &PgTxn,
        organization_ids: Vec<OrganizationId>,
        visibility: &Visibility,
    ) -> ReadTenancyResult<Self> {
        let mut billing_account_ids = Vec::with_capacity(organization_ids.len());
        for organization_id in &organization_ids {
            let row = txn
                .query_opt(GET_ORGANIZATION, &[organization_id, visibility])
                .await?
                .ok_or(ReadTenancyError::BillingAccountNotFoundForOrganization(
                    *organization_id,
                ))?;
            let billing_account_id = row.try_get("billing_account_id")?;
            billing_account_ids.push(billing_account_id);
        }
        Ok(Self {
            universal: true,
            billing_account_ids,
            organization_ids,
            workspace_ids: Vec::new(),
        })
    }

    pub async fn new_workspace(
        txn: &PgTxn,
        workspace_ids: Vec<WorkspaceId>,
        visibility: &Visibility,
    ) -> ReadTenancyResult<Self> {
        let mut organization_ids = Vec::new();
        let mut billing_account_ids = Vec::new();

        for workspace_id in &workspace_ids {
            let row = txn
                .query_opt(GET_WORKSPACE, &[workspace_id, visibility])
                .await?
                .ok_or(ReadTenancyError::WorkspaceNotFound(*workspace_id))?;
            let organization_id = row.try_get("organization_id")?;
            organization_ids.push(organization_id);
            let billing_account_id = row.try_get("billing_account_id")?;
            billing_account_ids.push(billing_account_id);
        }
        Ok(Self {
            organization_ids,
            billing_account_ids,
            workspace_ids,
            universal: true,
        })
    }

    pub fn into_local(mut self) -> Self {
        self.universal = false;
        self
    }
}

impl postgres_types::ToSql for ReadTenancy {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}
