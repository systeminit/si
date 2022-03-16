use serde::{Deserialize, Serialize};
use si_data::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    BillingAccountId, OrganizationId, ReadTenancy, ReadTenancyError, Tenancy, WorkspaceId,
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
    pub fn into_universal(mut self) -> Self {
        self.universal = true;
        self
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
        txn: &PgTxn<'_>,
    ) -> Result<ReadTenancy, ReadTenancyError> {
        ReadTenancy::try_from_tenancy(txn, self.into()).await
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

impl From<&WriteTenancy> for Tenancy {
    fn from(from: &WriteTenancy) -> Self {
        Self {
            universal: from.universal,
            billing_account_ids: from.billing_account_ids.clone(),
            organization_ids: from.organization_ids.clone(),
            workspace_ids: from.workspace_ids.clone(),
        }
    }
}

// This tecnically allow us to bypass WriteTenancy limitation, but it's only for interoperability until Tenancy dies
// But in practice we don't use Tenancy in a way that should bypass it
impl From<&Tenancy> for WriteTenancy {
    fn from(from: &Tenancy) -> Self {
        Self {
            universal: from.universal,
            billing_account_ids: from.billing_account_ids.clone(),
            organization_ids: from.organization_ids.clone(),
            workspace_ids: from.workspace_ids.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_tenancy() {
        assert_eq!(
            Tenancy::from(&WriteTenancy::new_workspace(1.into())),
            Tenancy::new_workspace(vec![1.into()])
        );
        assert_eq!(
            Tenancy::from(&WriteTenancy::new_organization(1.into())),
            Tenancy::new_organization(vec![1.into()])
        );
        assert_eq!(
            Tenancy::from(&WriteTenancy::new_billing_account(1.into())),
            Tenancy::new_billing_account(vec![1.into()])
        );
        assert_eq!(
            Tenancy::from(&WriteTenancy::new_workspace(1.into()).into_universal()),
            Tenancy::new_workspace(vec![1.into()]).into_universal()
        );
        assert_eq!(
            Tenancy::from(&WriteTenancy::new_organization(1.into()).into_universal()),
            Tenancy::new_organization(vec![1.into()]).into_universal()
        );
        assert_eq!(
            Tenancy::from(&WriteTenancy::new_billing_account(1.into()).into_universal()),
            Tenancy::new_billing_account(vec![1.into()]).into_universal()
        );
    }

    #[test]
    fn from_tenancy() {
        assert_eq!(
            WriteTenancy::new_workspace(1.into()),
            WriteTenancy::from(&Tenancy::new_workspace(vec![1.into()]))
        );
        assert_eq!(
            WriteTenancy::new_organization(1.into()),
            WriteTenancy::from(&Tenancy::new_organization(vec![1.into()]))
        );
        assert_eq!(
            WriteTenancy::new_billing_account(1.into()),
            WriteTenancy::from(&Tenancy::new_billing_account(vec![1.into()]))
        );
        assert_eq!(
            WriteTenancy::new_workspace(1.into()).into_universal(),
            WriteTenancy::from(&Tenancy::new_workspace(vec![1.into()]).into_universal())
        );
        assert_eq!(
            WriteTenancy::new_organization(1.into()).into_universal(),
            WriteTenancy::from(&Tenancy::new_organization(vec![1.into()]).into_universal())
        );
        assert_eq!(
            WriteTenancy::new_billing_account(1.into()).into_universal(),
            WriteTenancy::from(&Tenancy::new_billing_account(vec![1.into()]).into_universal())
        );
    }
}
