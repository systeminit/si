use crate::func::FuncId;
use crate::{DalContext, PropId, SchemaVariantId};
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
    #[serde(flatten)]
    context: AttributeContext,
    /// Name for [`Self`] that can be used for identification.
    name: Option<String>,
    /// Indicates if this socket is only for internal use.
    internal_only: bool,
    /// Definition of the output type.
    type_definition: Option<String>,
    /// Source [`Prop`] for the socket.
    source_prop_id: PropId,
    /// Transformation [`Func`] for the socket.
    transformation_func_id: FuncId,
}

impl OutputSocket {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        context: AttributeContext,
        name: Option<String>,
        internal_only: bool,
        source_prop_id: PropId,
        transformation_func_id: FuncId,
    ) -> OutputSocketResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM output_socket_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &context,
                    &name,
                    &internal_only,
                    &source_prop_id,
                    &transformation_func_id,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(name, Option<String>, OutputSocketResult);
    standard_model_accessor_ro!(internal_only, bool);
    standard_model_accessor!(type_definition, Option<String>, OutputSocketResult);
    standard_model_accessor!(source_prop_id, Pk(PropId), OutputSocketResult);
    standard_model_accessor!(transformation_func_id, Pk(FuncId), OutputSocketResult);

    /// Find all output sockets for a given [`SchemaVariant`].
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_schema_variant(
        ctx: &DalContext<'_, '_>,
        schema_variant_id: SchemaVariantId,
    ) -> OutputSocketResult<Vec<Self>> {
        // FIXME(nick): make real query.
        let rows = ctx
            .txns()
            .pg()
            .query(
                "foo",
                &[ctx.read_tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }
}
