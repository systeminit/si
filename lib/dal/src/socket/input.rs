use crate::{DalContext, PropId, SchemaId, SchemaVariantId};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    HistoryEventError, StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
};

const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/input_socket_list_for_schema_variant.sql");

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

    /// Indicates which [`Prop`] this socket belongs to.
    prop_id: PropId,
    /// Indicates which [`Schema`] this socket belongs to.
    schema_id: SchemaId,
    /// Indicates which [`SchemaVariant`] this socket belongs to.
    schema_variant_id: SchemaVariantId,
    /// Name for [`Self`] that can be used for identification.
    name: Option<String>,
    /// Indicates if this socket is only for internal use.
    internal_only: bool,
    /// Definition of the input type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
}

impl InputSocket {
    #[tracing::instrument(skip(ctx))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        name: Option<String>,
        internal_only: bool,
        type_definition: Option<String>,
    ) -> InputSocketResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM input_socket_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &prop_id,
                    &schema_id,
                    &schema_variant_id,
                    &name,
                    &internal_only,
                    &type_definition,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(name, Option<String>, InputSocketResult);
    standard_model_accessor_ro!(internal_only, bool);
    standard_model_accessor!(type_definition, Option<String>, InputSocketResult);

    standard_model_accessor_ro!(prop_id, PropId);
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

    /// Find all input sockets for a given [`SchemaVariant`].
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_schema_variant(
        ctx: &DalContext<'_, '_>,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_SCHEMA_VARIANT,
                &[ctx.read_tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }
}
