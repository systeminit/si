use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::binding::FuncBindingId;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::provider::emit;
use crate::provider::emit::EmitError;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    AttributePrototype, AttributePrototypeError, ComponentId, FuncId, HistoryEventError,
    InternalProviderId, StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
};
use crate::{
    AttributeContext, AttributeContextBuilderError, AttributeContextError, AttributePrototypeId,
    AttributeValue, DalContext, SchemaId, SchemaVariantId,
};

const LIST_FOR_ATTRIBUTE_PROTOTYPE_WITH_HEAD: &str =
    include_str!("../queries/external_provider_list_for_attribute_prototype_with_head.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/external_provider_list_for_schema_variant.sql");
const LIST_FROM_INTERNAL_PROVIDER_USE: &str =
    include_str!("../queries/external_provider_list_from_internal_provider_use.sql");

#[derive(Error, Debug)]
pub enum ExternalProviderError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("emit error: {0}")]
    Emit(#[from] EmitError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("unexpected: attribute prototype field is empty")]
    EmptyAttributePrototype,
    #[error(
        "provided attribute context does not specify an internal provider id (required for emit)"
    )]
    MissingInternalProviderForEmit,
    #[error("not found for id: {0}")]
    NotFound(ExternalProviderId),
    #[error("schema id mismatch: {0} (self) and {1} (provided)")]
    SchemaMismatch(SchemaId, SchemaId),
    #[error("schema variant id mismatch: {0} (self) and {1} (provided)")]
    SchemaVariantMismatch(SchemaVariantId, SchemaVariantId),
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

/// This provider can only provide data to external [`SchemaVariants`](crate::SchemaVariant). It can
/// only consume data within its own [`SchemaVariant`](crate::SchemaVariant).
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

    /// Indicates which [`Schema`](crate::Schema) this provider belongs to.
    schema_id: SchemaId,
    /// Indicates which [`SchemaVariant`](crate::SchemaVariant) this provider belongs to.
    schema_variant_id: SchemaVariantId,
    /// Indicates which transformation function should be used during [`Self::emit()`].
    attribute_prototype_id: Option<AttributePrototypeId>,

    /// Name for [`Self`] that can be used for identification.
    name: Option<String>,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
}

impl ExternalProvider {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        name: Option<String>,
        type_definition: Option<String>,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
    ) -> ExternalProviderResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM external_provider_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &schema_id,
                    &schema_variant_id,
                    &name,
                    &type_definition,
                ],
            )
            .await?;

        let mut external_provider: ExternalProvider =
            standard_model::finish_create_from_row(ctx, row).await?;

        let attribute_prototype = AttributePrototype::new(
            ctx,
            func_id,
            func_binding_id,
            func_binding_return_value_id,
            external_provider.attribute_context()?,
            None,
            None,
        )
        .await?;
        external_provider
            .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
            .await?;

        Ok(external_provider)
    }

    // Immutable fields.
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

    // Mutable fields.
    standard_model_accessor!(name, Option<String>, ExternalProviderResult);
    standard_model_accessor!(type_definition, Option<String>, ExternalProviderResult);
    standard_model_accessor!(
        attribute_prototype_id,
        OptionBigInt<AttributePrototypeId>,
        ExternalProviderResult
    );

    /// Evaluate with a provided [`AttributeContext`](crate::AttributeContext) and return the
    /// resulting [`AttributeValue`](crate::AttributeValue).
    ///
    /// Requirements for the provided [`AttributeContext`](crate::AttributeContext):
    /// - The least specific field set must be the [`InternalProviderId`](crate::InternalProvider)
    ///   field (which _should_ correspond to an "explicit"
    ///   [`InternalProvider`](crate::InternalProvider))
    /// - If the [`SchemaId`](crate::Schema) is set, it must match the corresponding field on
    ///   [`Self`]
    /// - If the [`SchemaVariantId`](crate::SchemaVariant) is set, it must match the corresponding
    ///   field on [`Self`]
    pub async fn emit(
        &self,
        ctx: &DalContext<'_, '_>,
        consume_attribute_context: AttributeContext,
    ) -> ExternalProviderResult<AttributeValue> {
        // Ensure that the least specific field in the provided context matches what we expect.
        if !consume_attribute_context.is_least_specific_field_kind_internal_provider()? {
            return Err(ExternalProviderError::MissingInternalProviderForEmit);
        }

        // Ensure that if the schema and/or schema variant fields are set, that they match our
        // corresponding fields because we consume internally.
        if !consume_attribute_context.is_schema_unset()
            && consume_attribute_context.schema_id() != self.schema_id
        {
            return Err(ExternalProviderError::SchemaMismatch(
                self.schema_id,
                consume_attribute_context.schema_id(),
            ));
        }
        if !consume_attribute_context.is_schema_variant_unset()
            && consume_attribute_context.schema_variant_id() != self.schema_variant_id
        {
            return Err(ExternalProviderError::SchemaVariantMismatch(
                self.schema_variant_id,
                consume_attribute_context.schema_variant_id(),
            ));
        }

        let emit_context = AttributeContextBuilder::from(consume_attribute_context)
            .unset_internal_provider_id()
            .set_external_provider_id(self.id)
            .to_context()?;

        let attribute_prototype_id = self
            .attribute_prototype_id
            .ok_or(ExternalProviderError::EmptyAttributePrototype)?;

        let emit_attribute_value = emit::emit(
            ctx,
            attribute_prototype_id,
            consume_attribute_context,
            emit_context,
        )
        .await?;
        Ok(emit_attribute_value)
    }

    /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
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

    /// Find all [`Self`] for a given [`AttributePrototypeId`](crate::AttributePrototype).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_attribute_prototype_with_head(
        ctx: &DalContext<'_, '_>,
        attribute_prototype_id: AttributePrototypeId,
        head_component_id: ComponentId,
    ) -> ExternalProviderResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_ATTRIBUTE_PROTOTYPE_WITH_HEAD,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &head_component_id,
                ],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find all [`Self`] that have
    /// [`AttributePrototypeArguments`](crate::AttributePrototypeArgument) referencing the provided
    /// [`InternalProviderId`](crate::InternalProvider).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_from_internal_provider_use(
        ctx: &DalContext<'_, '_>,
        internal_provider_id: InternalProviderId,
    ) -> ExternalProviderResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FROM_INTERNAL_PROVIDER_USE,
                &[ctx.read_tenancy(), ctx.visibility(), &internal_provider_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Returns an [`AttributeContext`](crate::AttributeContext) corresponding to our id, our
    /// [`SchemaId`](crate::SchemaId) and our [`SchemaVariantId`](crate::SchemaVariantId).
    pub fn attribute_context(&self) -> ExternalProviderResult<AttributeContext> {
        Ok(AttributeContext::builder()
            .set_external_provider_id(self.id)
            .set_schema_id(self.schema_id)
            .set_schema_variant_id(self.schema_variant_id)
            .to_context()?)
    }
}
