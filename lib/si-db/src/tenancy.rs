use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgTxn;
use si_id::WorkspacePk;
use telemetry::prelude::*;

use crate::{
    SiDbError,
    SiDbResult,
};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tenancy {
    #[serde(rename = "tenancy_workspace_pk")]
    workspace_pk: Option<WorkspacePk>,
}

impl Tenancy {
    pub fn new(workspace_pk: WorkspacePk) -> Self {
        Self {
            workspace_pk: Some(workspace_pk),
        }
    }

    pub fn new_empty() -> Self {
        Self { workspace_pk: None }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn check(&self, txn: &PgTxn, tenancy: &Tenancy) -> SiDbResult<bool> {
        let row = txn
            .query_one(
                "SELECT in_tenancy_v1($1::jsonb, $2::ident) AS result",
                &[tenancy, &self.workspace_pk],
            )
            .await?;
        let result = row.try_get("result")?;
        Ok(result)
    }

    pub fn workspace_pk(&self) -> SiDbResult<WorkspacePk> {
        self.workspace_pk.ok_or(SiDbError::NoWorkspace)
    }

    pub fn workspace_pk_opt(&self) -> Option<WorkspacePk> {
        self.workspace_pk
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let workspace_pk = self.workspace_pk.unwrap_or(WorkspacePk::NONE);
        workspace_pk.to_string().as_bytes().to_vec()
    }
}

impl From<WorkspacePk> for Tenancy {
    fn from(workspace_pk: WorkspacePk) -> Self {
        Self::new(workspace_pk)
    }
}

impl postgres_types::ToSql for Tenancy {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> std::result::Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
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
    ) -> std::result::Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}
