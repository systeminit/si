use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, standard_model_accessor_ro, DalContext, HistoryEvent, HistoryEventError, OrganizationPk,
    Timestamp, TransactionsError,
};

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Transactions(#[from] Box<TransactionsError>),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

pk!(WorkspacePk);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pk: WorkspacePk,
    organization_pk: OrganizationPk,
    name: String,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl Workspace {
    pub fn pk(&self) -> &WorkspacePk {
        &self.pk
    }

    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        organization_pk: OrganizationPk,
    ) -> WorkspaceResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM workspace_create_v1($1, $2)",
                &[&name, &organization_pk],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        // Ensures HistoryEvent gets stored in our workspace
        let ctx = ctx
            .clone_with_new_workspace_tenancies(object.pk)
            .await
            .map_err(Box::new)?;
        let _history_event = HistoryEvent::new(
            &ctx,
            "organization.create".to_owned(),
            "Organization created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor_ro!(name, String);
}
