use serde::{Deserialize, Serialize};
use si_data::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{BillingAccountId, OrganizationId, WorkspaceId};

#[derive(Error, Debug)]
pub enum TenancyError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

pub type TenancyResult<T> = Result<T, TenancyError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Tenancy {
    #[serde(rename = "tenancy_universal")]
    pub universal: bool,
    #[serde(rename = "tenancy_billing_account_ids")]
    pub billing_account_ids: Vec<BillingAccountId>,
    #[serde(rename = "tenancy_organization_ids")]
    pub organization_ids: Vec<OrganizationId>,
    #[serde(rename = "tenancy_workspace_ids")]
    pub workspace_ids: Vec<WorkspaceId>,
}

impl Tenancy {
    pub fn new(
        universal: bool,
        billing_account_ids: Vec<BillingAccountId>,
        organization_ids: Vec<OrganizationId>,
        workspace_ids: Vec<WorkspaceId>,
    ) -> Self {
        Self {
            universal,
            billing_account_ids,
            organization_ids,
            workspace_ids,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            universal: false,
            billing_account_ids: Vec::new(),
            organization_ids: Vec::new(),
            workspace_ids: Vec::new(),
        }
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
            universal: false,
            billing_account_ids,
            organization_ids: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_organization(organization_ids: Vec<OrganizationId>) -> Self {
        Self {
            universal: false,
            billing_account_ids: Vec::new(),
            organization_ids,
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_workspace(workspace_ids: Vec<WorkspaceId>) -> Self {
        Self {
            universal: false,
            billing_account_ids: Vec::new(),
            organization_ids: Vec::new(),
            workspace_ids,
        }
    }

    #[instrument(skip_all)]
    pub async fn check(&self, txn: &PgTxn<'_>, read_tenancy: &Tenancy) -> TenancyResult<bool> {
        let row = txn
            .query_one(
                "SELECT result FROM in_tenancy_v1($1, $2, $3, $4, $5)",
                &[
                    read_tenancy,
                    &self.universal,
                    &self.billing_account_ids,
                    &self.organization_ids,
                    &self.workspace_ids,
                ],
            )
            .await?;
        let result = row.try_get("result")?;
        Ok(result)
    }
}

impl postgres_types::ToSql for Tenancy {
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
