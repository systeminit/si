use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{BillingAccountPk, OrganizationPk, StandardModelError, WorkspaceError, WorkspacePk};

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
    WorkspaceNotFound(WorkspacePk),
    #[error("organization not found for workspace error: {0}")]
    OrganizationNotFoundForWorkspace(WorkspacePk),
    #[error("billing account not found for organization error: {0}")]
    BillingAccountNotFoundForOrganization(OrganizationPk),
}

pub type ReadTenancyResult<T> = Result<T, ReadTenancyError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ReadTenancy {
    #[serde(rename = "tenancy_workspace_pk")]
    workspace_pk: Option<WorkspacePk>,
}

impl ReadTenancy {
    pub fn new(
        txn: &PgTxn,
        workspace_pk: Option<WorkspacePk>,
    ) ->Self {
        Self {
            workspace_pk,
        }
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
