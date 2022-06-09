use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::{AttributeContextBuilder, UNSET_ID_VALUE};
use crate::func::binding::{FuncBindingError, FuncBindingId};
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::provider::emit;
use crate::provider::emit::EmitError;
use crate::standard_model::object_option_from_row_option;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    AttributeContextBuilderError, AttributePrototype, AttributePrototypeError,
    AttributePrototypeId, FuncId, HistoryEventError, Prop, StandardModel, StandardModelError,
    Timestamp, Visibility, WriteTenancy,
};
use crate::{
    AttributeContext, AttributeContextError, AttributeValue, DalContext, Func, FuncBinding, PropId,
    SchemaId, SchemaVariantId,
};

const GET_FOR_PROP: &str = include_str!("../queries/internal_provider_get_for_prop.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/internal_provider_list_for_schema_variant.sql");
const LIST_FOR_ATTRIBUTE_PROTOTYPE: &str =
    include_str!("../queries/internal_provider_list_for_attribute_prototype.sql");

#[derive(Error, Debug)]
pub enum InternalProviderError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("emit error: {0}")]
    Emit(#[from] EmitError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("unexpected: attribute prototype field is empty")]
    EmptyAttributePrototype,
    #[error("provided attribute context does not specify an ExternalProviderId (required for explicit emit")]
    MissingExternalProviderForExplicitEmit,
    #[error("missing func")]
    MissingFunc(String),
    #[error("provided attribute context does not specify a PropId (required for implicit emit)")]
    MissingPropForImplicitEmit,
    #[error("not found for id: {0}")]
    NotFound(InternalProviderId),
    #[error("prop not found for id: {0}")]
    PropNotFound(PropId),
    #[error("schema id mismatch: {0} (self) and {1} (provided)")]
    SchemaMismatch(SchemaId, SchemaId),
    #[error("schema variant id mismatch: {0} (self) and {1} (provided)")]
    SchemaVariantMismatch(SchemaVariantId, SchemaVariantId),
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

/// This provider can only provide data within its own [`SchemaVariant`](crate::SchemaVariant).
///
/// If the "internal_consumer" field is set to "true", this provider can only consume data from within
/// its own [`SchemaVariant`](crate::SchemaVariant). Internally-consuming [`InternalProviders`](Self)
/// are called "implicit" [`InternalProviders`](Self)
///
/// If the "internal_consumer" field is set to "false", this provider can only consume data from other
/// [`SchemaVariants`](crate::SchemaVariant). Externally-consuming [`InternalProviders`](Self)
/// are called "explicit" [`InternalProviders`](Self).
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

    /// Indicates which [`Prop`](crate::Prop) this provider belongs to. This will be
    /// [`UNSET_ID_VALUE`](crate::attribute::context::UNSET_ID_VALUE) if [`Self`] is "explicit". If
    /// [`Self`] is "implicit", this will always be a "set" id.
    prop_id: PropId,
    /// Indicates which [`Schema`](crate::Schema) this provider belongs to.
    schema_id: SchemaId,
    /// Indicates which [`SchemaVariant`](crate::SchemaVariant) this provider belongs to.
    schema_variant_id: SchemaVariantId,
    /// Indicates which transformation function should be used during [`Self::emit()`].
    attribute_prototype_id: Option<AttributePrototypeId>,

    /// Name for [`Self`] that can be used for identification.
    name: Option<String>,
    /// If this field is set to "true", the provider can only consume data for its corresponding
    /// function from within its own [`SchemaVariant`](crate::SchemaVariant). In this case, [`Self`]
    /// is "implicit".
    ///
    /// If this field field is set to "false", the provider
    /// can only consume data from other [`SchemaVariants`](crate::SchemaVariant). In this case,
    /// [`Self`] is "explicit".
    internal_consumer: bool,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    outbound_type_definition: Option<String>,
}

impl InternalProvider {
    #[tracing::instrument(skip(ctx))]
    pub async fn new_implicit(
        ctx: &DalContext<'_, '_>,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> InternalProviderResult<Self> {
        // Use the prop name for the implicit internal provider name. We need an owned string that
        // we then borrow for the query.
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or(InternalProviderError::PropNotFound(prop_id))?;
        let name = prop.name().to_string();

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
                    &true,
                    &Option::<String>::None,
                    &Option::<String>::None,
                ],
            )
            .await?;
        let mut internal_provider: InternalProvider =
            standard_model::finish_create_from_row(ctx, row).await?;

        let identity_func_name = "si:identity".to_string();
        let identity_func: Func = Func::find_by_attr(ctx, "name", &identity_func_name)
            .await?
            .pop()
            .ok_or(InternalProviderError::MissingFunc(identity_func_name))?;
        let (identity_func_binding, identity_func_binding_return_value) =
            FuncBinding::find_or_create_and_execute(
                ctx,
                serde_json::json![{ "identity": null }],
                *identity_func.id(),
            )
            .await?;

        let context = AttributeContext::builder()
            .set_internal_provider_id(*internal_provider.id())
            .set_schema_id(schema_id)
            .set_schema_variant_id(schema_variant_id)
            .to_context()?;

        // Key and parent are unneeded because the provider exists not strictly as part of the
        // schema values _and_ because implicit internal providers cannot be created for descendants
        // of maps and arrays.
        let attribute_prototype = AttributePrototype::new(
            ctx,
            *identity_func.id(),
            *identity_func_binding.id(),
            *identity_func_binding_return_value.id(),
            context,
            None,
            None,
        )
        .await?;

        internal_provider
            .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
            .await?;
        Ok(internal_provider)
    }

    #[tracing::instrument(skip(ctx))]
    pub async fn new_explicit(
        ctx: &DalContext<'_, '_>,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        name: Option<String>,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
    ) -> InternalProviderResult<Self> {
        let prop_id: PropId = UNSET_ID_VALUE.into();
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
                    &false,
                    &Option::<String>::None,
                    &Option::<String>::None,
                ],
            )
            .await?;

        let mut explicit_internal_provider: InternalProvider =
            standard_model::finish_create_from_row(ctx, row).await?;

        let attribute_prototype = AttributePrototype::new(
            ctx,
            func_id,
            func_binding_id,
            func_binding_return_value_id,
            explicit_internal_provider.attribute_context()?,
            None,
            None,
        )
        .await?;
        explicit_internal_provider
            .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
            .await?;

        Ok(explicit_internal_provider)
    }

    // Immutable fields.
    standard_model_accessor_ro!(prop_id, PropId);
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);
    standard_model_accessor_ro!(internal_consumer, bool);

    // Mutable fields.
    standard_model_accessor!(
        attribute_prototype_id,
        OptionBigInt<AttributePrototypeId>,
        InternalProviderResult
    );
    standard_model_accessor!(name, Option<String>, InternalProviderResult);
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

    /// Evaluate with a provided [`AttributeContext`](crate::AttributeContext) and return the
    /// resulting [`AttributeValue`](crate::AttributeValue).
    ///
    /// Requirements for the provided [`AttributeContext`](crate::AttributeContext):
    /// - If we are implicit, the provided context must specify a [`PropId`](crate::Prop)
    /// - If we are explicit, the provided context must specify an
    ///   [`ExternalProviderId`](crate::ExternalProvider)
    /// - If the [`SchemaId`](crate::Schema) is set, it must match the corresponding field on
    ///   [`Self`]
    /// - If the [`SchemaVariantId`](crate::SchemaVariant) is set, it must match the corresponding
    ///   field on [`Self`]
    pub async fn emit(
        &self,
        ctx: &DalContext<'_, '_>,
        attribute_context: AttributeContext,
    ) -> InternalProviderResult<AttributeValue> {
        // Ensure that the least specific field in the provided context matches what we expect.
        if self.internal_consumer && !attribute_context.is_least_specific_field_kind_prop()? {
            return Err(InternalProviderError::MissingPropForImplicitEmit);
        } else if !self.internal_consumer
            && !attribute_context.is_least_specific_field_kind_external_provider()?
        {
            return Err(InternalProviderError::MissingExternalProviderForExplicitEmit);
        }

        // Ensure that if the schema and/or schema variant fields are set, that they match our
        // corresponding fields. We only need to perform this check for internal consumers.
        if self.internal_consumer {
            if !attribute_context.is_schema_unset()
                && attribute_context.schema_id() != self.schema_id
            {
                return Err(InternalProviderError::SchemaMismatch(
                    self.schema_id,
                    attribute_context.schema_id(),
                ));
            }
            if !attribute_context.is_schema_variant_unset()
                && attribute_context.schema_variant_id() != self.schema_variant_id
            {
                return Err(InternalProviderError::SchemaVariantMismatch(
                    self.schema_variant_id,
                    attribute_context.schema_variant_id(),
                ));
            }
        }

        // Update or create the emit attribute value using the newly generated func binding return
        // value. For its context, we use the provided context and replace the least specific field
        // with our own InternalProviderId.
        let emit_context = AttributeContextBuilder::from(attribute_context)
            .unset_prop_id()
            .unset_external_provider_id()
            .set_internal_provider_id(self.id)
            .to_context()?;

        let attribute_prototype_id = self
            .attribute_prototype_id
            .ok_or(InternalProviderError::EmptyAttributePrototype)?;

        let emit_attribute_value =
            emit::emit(ctx, attribute_prototype_id, attribute_context, emit_context).await?;
        Ok(emit_attribute_value)
    }

    /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
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

    /// Find all [`Self`] for a given [`AttributePrototypeId`](crate::AttributePrototype).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_attribute_prototype(
        ctx: &DalContext<'_, '_>,
        attribute_prototype_id: AttributePrototypeId,
    ) -> InternalProviderResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_ATTRIBUTE_PROTOTYPE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                ],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Returns an [`AttributeContext`](crate::AttributeContext) corresponding to our id, our
    /// [`SchemaId`](crate::SchemaId) and our [`SchemaVariantId`](crate::SchemaVariantId).
    pub fn attribute_context(&self) -> InternalProviderResult<AttributeContext> {
        Ok(AttributeContext::builder()
            .set_internal_provider_id(self.id)
            .set_schema_id(self.schema_id)
            .set_schema_variant_id(self.schema_variant_id)
            .to_context()?)
    }

    /// Gets [`Self`] for a given [`PropId`](crate::Prop). This will only work for
    /// implicit [`InternalProviders`](Self).
    pub async fn get_for_prop(
        ctx: &DalContext<'_, '_>,
        prop_id: PropId,
    ) -> InternalProviderResult<Option<Self>> {
        let row = ctx
            .pg_txn()
            .query_opt(
                GET_FOR_PROP,
                &[ctx.read_tenancy(), ctx.visibility(), &prop_id],
            )
            .await?;
        Ok(object_option_from_row_option(row)?)
    }
}
