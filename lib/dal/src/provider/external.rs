use crate::{AttributePrototypeId, DalContext, PropId, SchemaId, SchemaVariantId};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    HistoryEventError, StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
};

const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/external_provider_list_for_schema_variant.sql");

#[derive(Error, Debug)]
pub enum ExternalProviderError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type ExternalProviderResult<T> = Result<T, ExternalProviderError>;

pk!(ExternalProviderPk);
pk!(ExternalProviderId);

impl_standard_model! {
    model: ExternalProvider,
    pk: ExternalProviderPk,
    id: ExternalProviderId,
    table_name: "external_providers",
    history_event_label_base: "external_provider",
    history_event_message_name: "External Provider"
}

/// This provider can only provide data to external [`SchemaVariant`]s. It can only consume data
/// within its own [`SchemaVariant`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ExternalProvider {
    pk: ExternalProviderPk,
    id: ExternalProviderId,
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
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
    /// The [`AttributePrototype`] of the transformation to perform for the socket.
    /// It includes the transformation function itself and where to get the arguments for the
    /// transformation function.
    attribute_prototype_id: AttributePrototypeId,
}

impl ExternalProvider {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        name: Option<String>,
        type_definition: Option<String>,
        attribute_prototype_id: AttributePrototypeId,
    ) -> ExternalProviderResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM external_provider_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
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

    standard_model_accessor_ro!(prop_id, PropId);
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

    standard_model_accessor!(name, Option<String>, ExternalProviderResult);
    standard_model_accessor!(type_definition, Option<String>, ExternalProviderResult);
    standard_model_accessor_ro!(attribute_prototype_id, AttributePrototypeId);

    /// Find all external providers for a given [`SchemaVariant`].
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_schema_variant(
        ctx: &DalContext<'_, '_>,
        schema_variant_id: SchemaVariantId,
    ) -> ExternalProviderResult<Vec<Self>> {
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
