use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    HistoryActor, HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp,
    Visibility, Workspace, WorkspaceId,
};

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type SystemResult<T> = Result<T, SystemError>;

pk!(SystemPk);
pk!(SystemId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct System {
    pk: SystemPk,
    id: SystemId,
    name: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: System,
    pk: SystemPk,
    id: SystemId,
    table_name: "systems",
    history_event_label_base: "system",
    history_event_message_name: "System"
}

impl System {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> SystemResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM system_create_v1($1, $2, $3)",
                &[tenancy, visibility, &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;

        Ok(object)
    }

    standard_model_accessor!(name, String, SystemResult);

    standard_model_belongs_to!(
        lookup_fn: workspace,
        set_fn: set_workspace,
        unset_fn: unset_workspace,
        table: "system_belongs_to_workspace",
        model_table: "workspaces",
        belongs_to_id: WorkspaceId,
        returns: Workspace,
        result: SystemResult,
    );
}
