use serde::{Deserialize, Serialize};

use crate::WorkspacePk;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ReadTenancy {
    #[serde(rename = "tenancy_workspace_pk")]
    workspace_pk: Option<WorkspacePk>,
}

impl ReadTenancy {
    pub fn new(workspace_pk: WorkspacePk) -> Self {
        Self {
            workspace_pk: Some(workspace_pk),
        }
    }

    pub fn new_empty() -> Self {
        Self { workspace_pk: None }
    }

    pub fn workspace_pk(&self) -> Option<WorkspacePk> {
        self.workspace_pk
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
