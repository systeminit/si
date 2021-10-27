use serde::{Deserialize, Serialize};
use si_data::{PgTxn, PgError};
use thiserror::Error;

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
    #[serde(rename = "tenancy_billing_account_pks")]
    pub billing_account_pks: Vec<i64>,
    #[serde(rename = "tenancy_organization_pks")]
    pub organization_pks: Vec<i64>,
    #[serde(rename = "tenancy_workspace_pks")]
    pub workspace_pks: Vec<i64>,
}

impl Tenancy {
    #[tracing::instrument]
    pub fn new_empty() -> Self {
        return Tenancy {
            universal: false,
            billing_account_pks: Vec::new(),
            organization_pks: Vec::new(),
            workspace_pks: Vec::new(),
        };
    }

    #[tracing::instrument]
    pub fn new_universal() -> Self {
        return Tenancy {
            universal: true,
            billing_account_pks: Vec::new(),
            organization_pks: Vec::new(),
            workspace_pks: Vec::new(),
        };
    }

    #[tracing::instrument]
    pub fn new_billing_account(billing_account_pks: Vec<i64>) -> Self {
        return Tenancy {
            universal: false,
            billing_account_pks,
            organization_pks: Vec::new(),
            workspace_pks: Vec::new(),
        };
    }

    #[tracing::instrument]
    pub fn new_organization(organization_pks: Vec<i64>) -> Self {
        return Tenancy {
            universal: false,
            billing_account_pks: Vec::new(),
            organization_pks,
            workspace_pks: Vec::new(),
        };
    }

    #[tracing::instrument]
    pub fn new_workspace(workspace_pks: Vec<i64>) -> Self {
        return Tenancy {
            universal: false,
            billing_account_pks: Vec::new(),
            organization_pks: Vec::new(),
            workspace_pks,
        };
    }

    #[tracing::instrument(skip(txn))]
    pub async fn check(&self, txn: &PgTxn<'_>, check_tenancy: &Tenancy) -> TenancyResult<bool> {
        let row = txn
            .query_one(
                "SELECT result FROM in_tenancy_v1($1, $2, $3, $4, $5)",
                &[
                    &check_tenancy,
                    &self.universal,
                    &self.billing_account_pks,
                    &self.organization_pks,
                    &self.workspace_pks,
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
        postgres_types::ToSql::to_sql(&json, &ty, out)
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
        postgres_types::ToSql::to_sql(&json, &ty, out)
    }
}
