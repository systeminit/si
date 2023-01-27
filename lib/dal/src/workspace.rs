use crate::WriteTenancy;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, DalContext,
    HistoryEventError, OrganizationPk, StandardModel, StandardModelError, Timestamp, Visibility,
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
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

pk!(WorkspacePk);
pk!(WorkspaceId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pk: WorkspacePk,
    id: WorkspaceId,
    name: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Workspace,
    pk: WorkspacePk,
    id: WorkspaceId,
    table_name: "workspaces",
    history_event_label_base: "workspace",
    history_event_message_name: "Workspace"
}

impl Workspace {
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
                "SELECT object FROM workspace_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    &ctx.visibility(),
                    &name,
                    &organization_pk,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, WorkspaceResult);
}
