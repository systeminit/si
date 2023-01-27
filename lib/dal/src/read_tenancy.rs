use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    BillingAccountPk, OrganizationPk, StandardModelError, Visibility, WorkspaceError, WorkspaceId,
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
    BillingAccountNotFoundForOrganization(OrganizationPk),
}

pub type ReadTenancyResult<T> = Result<T, ReadTenancyError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ReadTenancy {
    #[serde(rename = "tenancy_billing_account_pks")]
    billing_account_pks: Vec<BillingAccountPk>,
    #[serde(rename = "tenancy_organization_pks")]
    organization_pks: Vec<OrganizationPk>,
    #[serde(rename = "tenancy_workspace_ids")]
    workspace_ids: Vec<WorkspaceId>,
}

impl ReadTenancy {
    pub fn billing_accounts(&self) -> &[BillingAccountPk] {
        &self.billing_account_pks
    }

    pub fn new_billing_account(billing_account_pks: Vec<BillingAccountPk>) -> Self {
        Self {
            billing_account_pks,
            organization_pks: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }

    pub async fn new_organization(
        txn: &PgTxn,
        organization_pks: Vec<OrganizationPk>,
    ) -> ReadTenancyResult<Self> {
        let mut billing_account_pks = Vec::with_capacity(organization_pks.len());
        for organization_pk in &organization_pks {
            let row = txn
                .query_opt(GET_ORGANIZATION, &[organization_pk])
                .await?
                .ok_or(ReadTenancyError::BillingAccountNotFoundForOrganization(
                    *organization_pk,
                ))?;
            let billing_account_pk = row.try_get("billing_account_pk")?;
            billing_account_pks.push(billing_account_pk);
        }
        Ok(Self {
            billing_account_pks,
            organization_pks,
            workspace_ids: Vec::new(),
        })
    }

    pub async fn new_workspace(
        txn: &PgTxn,
        workspace_ids: Vec<WorkspaceId>,
        visibility: &Visibility,
    ) -> ReadTenancyResult<Self> {
        let mut organization_pks = Vec::new();
        let mut billing_account_pks = Vec::new();

        for workspace_id in &workspace_ids {
            let row = txn
                .query_opt(GET_WORKSPACE, &[workspace_id, visibility])
                .await?
                .ok_or(ReadTenancyError::WorkspaceNotFound(*workspace_id))?;
            let organization_pk = row.try_get("organization_pk")?;
            organization_pks.push(organization_pk);
            let billing_account_pk = row.try_get("billing_account_pk")?;
            billing_account_pks.push(billing_account_pk);
        }
        Ok(Self {
            organization_pks,
            billing_account_pks,
            workspace_ids,
        })
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
