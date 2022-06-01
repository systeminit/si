use serde::{Deserialize, Serialize};
use si_data::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    BillingAccountId, DalContext, OrganizationId, ReadTenancy, ReadTenancyError, WorkspaceId,
};

#[derive(Error, Debug)]
pub enum WriteTenancyError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

pub type WriteTenancyResult<T> = Result<T, WriteTenancyError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct WriteTenancy {
    #[serde(rename = "tenancy_universal")]
    universal: bool,
    #[serde(rename = "tenancy_billing_account_ids")]
    billing_account_ids: Vec<BillingAccountId>,
    #[serde(rename = "tenancy_organization_ids")]
    organization_ids: Vec<OrganizationId>,
    #[serde(rename = "tenancy_workspace_ids")]
    workspace_ids: Vec<WorkspaceId>,
}

impl WriteTenancy {
    pub fn billing_accounts(&self) -> &[BillingAccountId] {
        &self.billing_account_ids
    }

    pub fn organizations(&self) -> &[OrganizationId] {
        &self.organization_ids
    }

    pub fn workspaces(&self) -> &[WorkspaceId] {
        &self.workspace_ids
    }

    pub fn universal(&self) -> bool {
        self.universal
    }

    pub fn into_universal(mut self) -> Self {
        self.universal = true;
        self
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

    pub fn new_billing_account(id: BillingAccountId) -> Self {
        Self {
            universal: false,
            billing_account_ids: vec![id],
            organization_ids: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_organization(id: OrganizationId) -> Self {
        Self {
            universal: false,
            billing_account_ids: Vec::new(),
            organization_ids: vec![id],
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_workspace(id: WorkspaceId) -> Self {
        Self {
            universal: false,
            billing_account_ids: Vec::new(),
            organization_ids: Vec::new(),
            workspace_ids: vec![id],
        }
    }

    #[instrument(skip_all)]
    pub async fn check(
        &self,
        txn: &PgTxn<'_>,
        read_tenancy: &ReadTenancy,
    ) -> WriteTenancyResult<bool> {
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

    pub async fn clone_into_read_tenancy(
        &self,
        ctx: &DalContext<'_, '_>,
    ) -> Result<ReadTenancy, ReadTenancyError> {
        let read_tenancy = if self.workspace_ids.is_empty() {
            if self.organization_ids.is_empty() {
                ReadTenancy::new_billing_account(self.billing_account_ids.clone())
            } else {
                ReadTenancy::new_organization(
                    ctx.txns().pg(),
                    self.organization_ids.clone(),
                    ctx.visibility(),
                )
                .await?
            }
        } else {
            ReadTenancy::new_workspace(
                ctx.txns().pg(),
                self.workspace_ids.clone(),
                ctx.visibility(),
            )
            .await?
        };
        Ok(read_tenancy)
    }
}

impl postgres_types::ToSql for WriteTenancy {
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
