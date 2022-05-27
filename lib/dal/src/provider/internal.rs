use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::backend::identity::FuncBackendIdentityArgs;
use crate::func::binding::FuncBindingError;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::standard_model::object_option_from_row_option;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    AttributeContextBuilderError, AttributePrototype, AttributePrototypeError,
    AttributePrototypeId, AttributeReadContext, AttributeValueError, AttributeView,
    FuncBackendKind, HistoryEventError, Prop, StandardModel, StandardModelError, Timestamp,
    Visibility, WriteTenancy,
};
use crate::{
    AttributeContext, AttributeContextError, AttributeValue, DalContext, Func, FuncBinding, PropId,
    SchemaId, SchemaVariantId,
};

const GET_FOR_PROP: &str = include_str!("../queries/internal_provider_get_for_prop.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/internal_provider_list_for_schema_variant.sql");

#[derive(Error, Debug)]
pub enum InternalProviderError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("unexpected: attribute prototype field is empty")]
    EmptyAttributePrototype,
    #[error("func not found")]
    FuncNotFound,
    #[error("missing attribute value")]
    MissingAttributeValue,
    #[error("missing attribute prototype")]
    MissingAttributePrototype,
    #[error("provided attribute context for internal consumer evaluation does not specify an ExternalProviderId")]
    MissingExternalProviderInAttributeContextForInternalConsumer,
    #[error("missing func")]
    MissingFunc(String),
    #[error(
        "provided attribute context for internal consumer evaluation does not specify a PropId"
    )]
    MissingPropForInternalConsumer,
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

/// This provider can only provide data within its own [`SchemaVariant`](crate::SchemaVariant). If
/// the "internal_consumer" field is set to "true", this provider can only consume data from within
/// its own [`SchemaVariant`](crate::SchemaVariant). If the "internal_consumer" field is set to
/// "false", this provider can only consume data from other
/// [`SchemaVariants`](crate::SchemaVariant).
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

    /// Indicates which [`Prop`](crate::Prop) this provider belongs to.
    prop_id: PropId,
    /// Indicates which [`Schema`](crate::Schema) this provider belongs to.
    schema_id: SchemaId,
    /// Indicates which [`SchemaVariant`](crate::SchemaVariant) this provider belongs to.
    schema_variant_id: SchemaVariantId,
    /// Indicates which transformation function should be used during evaluation. This should only
    /// be [`None`] within [`Self::new_implicit()`].
    attribute_prototype_id: Option<AttributePrototypeId>,

    /// Name for [`Self`] that can be used for identification.
    name: Option<String>,
    /// If this field is set to "true", the provider can only consume data for its corresponding
    /// function from within its own [`SchemaVariant`](crate::SchemaVariant). The corresponding
    /// context will have [`Prop`](crate::Prop) at the least specific field.
    ///
    /// If this field field is set to "false", the provider
    /// can only consume data from other [`SchemaVariants`](crate::SchemaVariant). The corresponding
    /// context will have [`ExternalProvider`](crate::ExternalProvider) at the least specific field.
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
                FuncBackendKind::Identity,
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

    standard_model_accessor_ro!(prop_id, PropId);
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);
    standard_model_accessor!(
        attribute_prototype_id,
        OptionBigInt<AttributePrototypeId>,
        InternalProviderResult
    );

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

    /// Gets [`Self`] for a given [`PropId`](crate::Prop).
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

    /// Evaluate with a given [`AttributeContext`](crate::AttributeContext) and return the
    /// resulting [`AttributeValue`](crate::AttributeValue). The provided context's fields must
    /// corresponding fields on [`Self`], if provided.
    pub async fn emit(
        &self,
        ctx: &DalContext<'_, '_>,
        attribute_context: AttributeContext,
    ) -> InternalProviderResult<AttributeValue> {
        // TODO(nick): remove this check once external consumers are supported.
        if !self.internal_consumer {
            todo!("externally consuming internal providers are not yet supported for evaluation");
        }

        // Ensure that the provided context contains a PropId for internal consumers and ensure that
        // the provided context contains an ExternalProviderId for external consumers.
        if self.internal_consumer && !attribute_context.is_least_specific_field_prop()? {
            return Err(InternalProviderError::MissingPropForInternalConsumer);
        } else if !self.internal_consumer
            && !attribute_context.is_least_specific_field_external_provider()?
        {
            return Err(
                InternalProviderError::MissingExternalProviderInAttributeContextForInternalConsumer,
            );
        }

        // If the Schema or SchemaVariant fields are set on the provided context, we need to check
        // if they match the corresponding fields of the InternalProvider.
        if !attribute_context.is_schema_unset() && attribute_context.schema_id() != self.schema_id {
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

        // Get the value by generating a view for the found attribute value.
        let found_attribute_value = AttributeValue::find_for_context(ctx, attribute_context.into())
            .await?
            .ok_or(InternalProviderError::MissingAttributeValue)?;
        let found_attribute_view_context = AttributeReadContext {
            prop_id: None,
            ..AttributeReadContext::from(attribute_context)
        };

        // TODO(nick,jacob): we do not need to generate an attribute view for externally consuming
        // internal providers.
        let found_attribute_view = AttributeView::new(
            ctx,
            found_attribute_view_context,
            Some(*found_attribute_value.id()),
        )
        .await?;
        let found_value = found_attribute_view.value().clone();

        // Generate a new func binding return value using the transformation function. Use the found value
        // from the attribute view as the args.
        let attribute_prototype_id = self
            .attribute_prototype_id
            .ok_or(InternalProviderError::EmptyAttributePrototype)?;
        let attribute_prototype = AttributePrototype::get_by_id(ctx, &attribute_prototype_id)
            .await?
            .ok_or(InternalProviderError::MissingAttributePrototype)?;
        let func = Func::get_by_id(ctx, &attribute_prototype.func_id())
            .await?
            .ok_or(InternalProviderError::FuncNotFound)?;

        let args: serde_json::Value = match func.backend_kind() {
            FuncBackendKind::Identity => serde_json::to_value(FuncBackendIdentityArgs {
                identity: found_value,
            })?,
            backend_kind => {
                todo!("emitting for backend kind {:?} not supported yet (currently, only internally consuming internal providers can emit)", backend_kind);
            }
        };

        let (func_binding, func_binding_return_value) =
            FuncBinding::find_or_create_and_execute(ctx, args, *func.id(), *func.backend_kind())
                .await?;

        // Update or create the emit attribute value using the newly generated func binding return
        // value. For its context, we use the provided context and replace the least specific field
        // with our own InternalProviderId.
        let emit_context = AttributeContextBuilder::from(attribute_context)
            .unset_prop_id()
            .unset_external_provider_id()
            .set_internal_provider_id(self.id)
            .to_context()?;

        let emit_attribute_value = if let Some(mut emit_attribute_value) =
            AttributeValue::find_for_context(ctx, emit_context.into()).await?
        {
            emit_attribute_value
                .set_func_binding_id(ctx, *func_binding.id())
                .await?;
            emit_attribute_value
                .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
                .await?;
            emit_attribute_value
        } else {
            AttributeValue::new(
                ctx,
                *func_binding.id(),
                *func_binding_return_value.id(),
                emit_context,
                Option::<String>::None,
            )
            .await?
        };

        Ok(emit_attribute_value)
    }

    /// Find all internal providers for a given [`SchemaVariant`](crate::SchemaVariant).
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
