use crate::func::FuncId;
use crate::{
    AttributePrototype, AttributePrototypeId, DalContext, PropId, SchemaId, SchemaVariantId,
};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    attribute::context::AttributeContext, impl_standard_model, pk, standard_model,
    standard_model_accessor, standard_model_accessor_ro, HistoryEventError, StandardModel,
    StandardModelError, Timestamp, Visibility, WriteTenancy,
};

const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/output_socket_list_for_schema_variant.sql");

#[derive(Error, Debug)]
pub enum OutputSocketError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type OutputSocketResult<T> = Result<T, OutputSocketError>;

pk!(OutputSocketPk);
pk!(OutputSocketId);

impl_standard_model! {
    model: OutputSocket,
    pk: OutputSocketPk,
    id: OutputSocketId,
    table_name: "output_sockets",
    history_event_label_base: "output_socket",
    history_event_message_name: "Output Socket"
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OutputSocket {
    pk: OutputSocketPk,
    id: OutputSocketId,
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
    /// Definition of the output type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
    /// The [`AttributePrototype`] of the transformation to perform for the socket.
    /// It includes the transformation function itself and where to get the arguments for the
    /// transformation function.
    attribute_prototype_id: AttributePrototypeId,
}

impl OutputSocket {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        name: Option<String>,
        internal_only: bool,
        type_definition: Option<String>,
        attribute_prototype_id: AttributePrototypeId,
    ) -> OutputSocketResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM output_socket_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &prop_id,
                    &schema_id,
                    &schema_variant_id,
                    &name,
                    &type_definition,
                    &attribute_prototype_id,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(name, Option<String>, OutputSocketResult);
    standard_model_accessor!(type_definition, Option<String>, OutputSocketResult);
    standard_model_accessor_ro!(attribute_prototype_id, AttributePrototypeId);

    standard_model_accessor_ro!(prop_id, PropId);
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

    /// Find all output sockets for a given [`SchemaVariant`].
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_schema_variant(
        ctx: &DalContext<'_, '_>,
        schema_variant_id: SchemaVariantId,
    ) -> OutputSocketResult<Vec<Self>> {
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
