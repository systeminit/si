use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    BillingAccountPk, DalContext, OrganizationPk, ReadTenancy, ReadTenancyError, WorkspaceId,
};

#[derive(Error, Debug)]
pub enum WriteTenancyError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

pub type WriteTenancyResult<T> = Result<T, WriteTenancyError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct WriteTenancy {
    #[serde(rename = "tenancy_billing_account_pks")]
    billing_account_pks: Vec<BillingAccountPk>,
    #[serde(rename = "tenancy_organization_pks")]
    organization_pks: Vec<OrganizationPk>,
    #[serde(rename = "tenancy_workspace_ids")]
    workspace_ids: Vec<WorkspaceId>,
}

impl WriteTenancy {
    pub fn billing_accounts(&self) -> &[BillingAccountPk] {
        &self.billing_account_pks
    }

    pub fn organizations(&self) -> &[OrganizationPk] {
        &self.organization_pks
    }

    pub fn workspaces(&self) -> &[WorkspaceId] {
        &self.workspace_ids
    }

    pub fn new_empty() -> Self {
        Self {
            billing_account_pks: Vec::new(),
            organization_pks: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_billing_account(bid: BillingAccountPk) -> Self {
        Self {
            billing_account_pks: vec![bid],
            organization_pks: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_organization(id: OrganizationPk) -> Self {
        Self {
            billing_account_pks: Vec::new(),
            organization_pks: vec![id],
            workspace_ids: Vec::new(),
        }
    }

    pub fn new_workspace(id: WorkspaceId) -> Self {
        Self {
            billing_account_pks: Vec::new(),
            organization_pks: Vec::new(),
            workspace_ids: vec![id],
        }
    }

    #[instrument(skip_all)]
    pub async fn check(&self, txn: &PgTxn, read_tenancy: &ReadTenancy) -> WriteTenancyResult<bool> {
        let row = txn
            .query_one(
                "SELECT in_tenancy_v1($1, $2, $3, $4) AS result",
                &[
                    read_tenancy,
                    &self.billing_account_pks,
                    &self.organization_pks,
                    &self.workspace_ids,
                ],
            )
            .await?;
        let result = row.try_get("result")?;
        Ok(result)
    }

    pub async fn into_read_tenancy(
        self,
        ctx: &DalContext,
    ) -> Result<ReadTenancy, ReadTenancyError> {
        let read_tenancy = if self.workspace_ids.is_empty() {
            if self.organization_pks.is_empty() {
                ReadTenancy::new_billing_account(self.billing_account_pks)
            } else {
                ReadTenancy::new_organization(ctx.txns().pg(), self.organization_pks).await?
            }
        } else {
            ReadTenancy::new_workspace(ctx.txns().pg(), self.workspace_ids, ctx.visibility())
                .await?
        };
        Ok(read_tenancy)
    }

    pub async fn clone_into_read_tenancy(
        &self,
        ctx: &DalContext,
    ) -> Result<ReadTenancy, ReadTenancyError> {
        self.clone().into_read_tenancy(ctx).await
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
