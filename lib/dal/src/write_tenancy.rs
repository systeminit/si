use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{ReadTenancy, WorkspacePk};

#[derive(Error, Debug)]
pub enum WriteTenancyError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

pub type WriteTenancyResult<T> = Result<T, WriteTenancyError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct WriteTenancy {
    #[serde(rename = "tenancy_workspace_pks")]
    workspace_pks: Vec<WorkspacePk>,
}

impl WriteTenancy {
    pub fn new_empty() -> Self {
        Self {
            workspace_pks: Vec::new(),
        }
    }

    pub fn new(workspace_pk: WorkspacePk) -> Self {
        Self {
            workspace_pks: vec![workspace_pk],
        }
    }

    #[instrument(skip_all)]
    pub async fn check(&self, txn: &PgTxn, read_tenancy: &ReadTenancy) -> WriteTenancyResult<bool> {
        let row = txn
            .query_one(
                "SELECT in_tenancy_v1($1::jsonb, $2::ident[]) AS result",
                &[read_tenancy, &self.workspace_pks],
            )
            .await?;
        let result = row.try_get("result")?;
        Ok(result)
    }

    pub fn into_read_tenancy(self) -> ReadTenancy {
        if let Some(pk) = self.workspace_pks.first() {
            ReadTenancy::new(*pk)
        } else {
            ReadTenancy::new_empty()
        }
    }

    pub fn clone_into_read_tenancy(&self) -> ReadTenancy {
        self.clone().into_read_tenancy()
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
