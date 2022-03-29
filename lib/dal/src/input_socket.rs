use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    attribute::context::AttributeContext, impl_standard_model, pk, standard_model,
    standard_model_accessor, standard_model_accessor_ro, HistoryEventError, StandardModel,
    StandardModelError, Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum InputSocketError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type InputSocketResult<T> = Result<T, InputSocketError>;

pk!(InputSocketPk);
pk!(InputSocketId);

impl_standard_model! {
    model: InputSocket,
    pk: InputSocketPk,
    id: InputSocketId,
    table_name: "input_sockets",
    history_event_label_base: "input_socket",
    history_event_message_name: "Input Socket"
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InputSocket {
    pk: InputSocketPk,
    id: InputSocketId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    context: AttributeContext,
    name: Option<String>,
    internal_only: bool,
    type_definition: Option<String>,
}

impl InputSocket {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        context: AttributeContext,
        name: Option<String>,
        internal_only: bool,
    ) -> InputSocketResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM input_socket_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &context,
                    &name,
                    &internal_only,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(name, Option<String>, InputSocketResult);
    standard_model_accessor_ro!(internal_only, bool);
    standard_model_accessor!(type_definition, Option<String>, InputSocketResult);
}
