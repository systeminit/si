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
    include_str!("../queries/internal_provider_list_for_schema_variant.sql");

#[derive(Error, Debug)]
pub enum InternalProviderError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type InternalProviderResult<T> = Result<T, InternalProviderError>;

pk!(InternalProviderPk);
pk!(InternalProviderId);

impl_standard_model! {
    model: InternalProvider,
    pk: InternalProviderPk,
    id: InternalProviderId,
    table_name: "internal_providers",
    history_event_label_base: "internal_provider",
    history_event_message_name: "Internal Provider"
}

/// This provider can only provide data within its own [`SchemaVariant`]. If the "internal_consumer"
/// field is set to "true", this provider can only consume data from within its own
/// [`SchemaVariant`]. If the "internal_consumer" field is set to "false", this provider can only
/// consume data from other [`SchemaVariant`]s.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InternalProvider {
    pk: InternalProviderPk,
    id: InternalProviderId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,

    /// Indicates which [`Prop`] this provider belongs to.
    prop_id: PropId,
    /// Indicates which [`Schema`] this provider belongs to.
    schema_id: SchemaId,
    /// Indicates which [`SchemaVariant`] this provider belongs to.
    schema_variant_id: SchemaVariantId,
    /// Name for [`Self`] that can be used for identification.
    name: Option<String>,
    /// If this field is set to "true", the provider can only consume data from within its own
    /// [`SchemaVariant`]. If this field field is set to "false", the provider can only consume data
    /// from other [`SchemaVariant`]s.
    internal_consumer: bool,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    outbound_type_definition: Option<String>,
}

impl InternalProvider {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        name: Option<String>,
        internal_consumer: bool,
        inbound_type_definition: Option<String>,
        outbound_type_definition: Option<String>,
    ) -> InternalProviderResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM internal_provider_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &prop_id,
                    &schema_id,
                    &schema_variant_id,
                    &name,
                    &internal_consumer,
                    &inbound_type_definition,
                    &outbound_type_definition,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor_ro!(prop_id, PropId);
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

    standard_model_accessor!(name, Option<String>, InternalProviderResult);
    standard_model_accessor_ro!(internal_consumer, bool);
    standard_model_accessor!(
        inbound_type_definition,
        Option<String>,
        InternalProviderResult
    );
    standard_model_accessor!(
        outbound_type_definition,
        Option<String>,
        InternalProviderResult
    );

    /// Find all internal providers for a given [`SchemaVariant`].
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_schema_variant(
        ctx: &DalContext<'_, '_>,
        schema_variant_id: SchemaVariantId,
    ) -> InternalProviderResult<Vec<Self>> {
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
