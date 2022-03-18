use serde::{Deserialize, Serialize};
use si_data::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    standard_model, BillingAccount, BillingAccountId, OrganizationId, StandardModel,
    StandardModelError, Tenancy, Workspace, WorkspaceError, WorkspaceId,
};

const GET_WORKSPACE: &str = include_str!("./queries/read_tenancy_get_workspace.sql");
const GET_BILLING_ACCOUNT: &str = include_str!("./queries/read_tenancy_get_billing_account.sql");

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
        txn: &PgTxn<'_>,
        organization_ids: Vec<OrganizationId>,
    ) -> ReadTenancyResult<Self> {
        let mut billing_account_ids = Vec::with_capacity(organization_ids.len());
        for organization_id in &organization_ids {
            let rows = txn.query(GET_BILLING_ACCOUNT, &[organization_id]).await?;
            let billing_accounts: Vec<BillingAccount> = standard_model::objects_from_rows(rows)?;

            if billing_accounts.is_empty() {
                return Err(ReadTenancyError::BillingAccountNotFoundForOrganization(
                    *organization_id,
                ));
            }
            for billing_account in billing_accounts {
                billing_account_ids.push(*billing_account.id());
            }
        }
        Ok(Self {
            universal: true,
            billing_account_ids,
            organization_ids,
            workspace_ids: Vec::new(),
        })
    }

    pub async fn new_workspace(
        txn: &PgTxn<'_>,
        workspace_ids: Vec<WorkspaceId>,
    ) -> ReadTenancyResult<Self> {
        let mut organization_ids = Vec::with_capacity(workspace_ids.len());

        for workspace_id in &workspace_ids {
            let row = txn.query_opt(GET_WORKSPACE, &[workspace_id]).await?;
            match standard_model::option_object_from_row::<Workspace>(row)? {
                None => return Err(ReadTenancyError::WorkspaceNotFound(*workspace_id)),
                Some(workspace) => {
                    let visibility = workspace.visibility();
                    if let Some(organization) = workspace.organization(txn, visibility).await? {
                        organization_ids.push(*organization.id());
                    } else {
                        return Err(ReadTenancyError::OrganizationNotFoundForWorkspace(
                            *workspace_id,
                        ));
                    }
                }
            }
        }

        let mut tenancy = Self::new_organization(txn, organization_ids).await?;
        tenancy.workspace_ids = workspace_ids;
        Ok(tenancy)
    }

    pub async fn try_from_tenancy(txn: &PgTxn<'_>, from: Tenancy) -> ReadTenancyResult<Self> {
        let mut read = Self::new_universal();
        if !from.workspace_ids.is_empty() {
            read = Self::new_workspace(txn, from.workspace_ids).await?;
        }
        if !from.organization_ids.is_empty() {
            let organization_ids = from
                .organization_ids
                .into_iter()
                .filter(|id| !read.organization_ids.contains(id))
                .collect();
            let org_read = Self::new_organization(txn, organization_ids).await?;
            read.organization_ids.extend(org_read.organization_ids);
            read.billing_account_ids
                .extend(org_read.billing_account_ids);
        }
        if !from.billing_account_ids.is_empty() {
            let billing_account_ids = from
                .billing_account_ids
                .into_iter()
                .filter(|id| !read.billing_account_ids.contains(id))
                .collect();
            let bill_read = Self::new_billing_account(billing_account_ids);
            read.billing_account_ids
                .extend(bill_read.billing_account_ids);
        }
        Ok(read)
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

impl From<&ReadTenancy> for Tenancy {
    fn from(from: &ReadTenancy) -> Self {
        Self {
            universal: from.universal,
            billing_account_ids: from.billing_account_ids.clone(),
            organization_ids: from.organization_ids.clone(),
            workspace_ids: from.workspace_ids.clone(),
        }
    }
}
