use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::{
    PgError,
    PgRow,
};
use si_id::WorkspacePk;
use thiserror::Error;

use crate::{
    DalContext,
    TransactionsError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceIntegrationsError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type WorkspaceIntegrationsResult<T> = Result<T, WorkspaceIntegrationsError>;

pub use si_id::WorkspaceIntegrationId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIntegration {
    pk: WorkspaceIntegrationId,
    workspace_pk: WorkspacePk,
    slack_webhook_url: Option<String>,
}

impl TryFrom<PgRow> for WorkspaceIntegration {
    type Error = WorkspaceIntegrationsError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            pk: row.try_get("pk")?,
            workspace_pk: row.try_get("workspace_pk")?,
            slack_webhook_url: row.try_get("slack_webhook_url")?,
        })
    }
}

impl WorkspaceIntegration {
    pub fn pk(&self) -> &WorkspaceIntegrationId {
        &self.pk
    }

    pub fn workspace_pk(&self) -> WorkspacePk {
        self.workspace_pk
    }

    pub fn slack_webhook_url(&self) -> Option<String> {
        self.slack_webhook_url.clone()
    }

    pub async fn update_webhook_url(
        &mut self,
        ctx: &DalContext,
        webhook_url: String,
    ) -> WorkspaceIntegrationsResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE workspace_integrations SET slack_webhook_url = $2 WHERE pk = $1",
                &[&self.pk, &webhook_url],
            )
            .await?;
        self.slack_webhook_url = Some(webhook_url);

        Ok(())
    }

    pub async fn new(
        ctx: &DalContext,
        webhook_url: Option<String>,
    ) -> WorkspaceIntegrationsResult<Self> {
        let workspace_pk = ctx.workspace_pk()?;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspace_integrations (workspace_pk, slack_webhook_url) VALUES ($1, $2)  RETURNING *",
                &[&workspace_pk, &webhook_url],
            )
            .await?;

        let workspace_integration = Self::try_from(row)?;

        Ok(workspace_integration)
    }

    pub async fn get_integrations_for_workspace_pk(
        ctx: &DalContext,
    ) -> WorkspaceIntegrationsResult<Option<Self>> {
        let workspace_pk = ctx.workspace_pk()?;

        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT * FROM workspace_integrations AS w WHERE workspace_pk = $1",
                &[&workspace_pk],
            )
            .await?;
        let maybe_workspace_integration = match maybe_row {
            Some(found) => Some(Self::try_from(found)?),
            None => None,
        };
        Ok(maybe_workspace_integration)
    }
}
