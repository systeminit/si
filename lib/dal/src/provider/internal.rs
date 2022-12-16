//! This module contains the concept of implicit and explicit [`InternalProviders`](InternalProvider).
//!
//! ## What are implicit [`InternalProviders`](InternalProvider)?
//!
//! Implicit [`InternalProviders`](InternalProvider) are created for every [`Prop`](crate::Prop) in
//! a [`SchemaVariant`](crate::SchemaVariant) that is not a descendant of a [`map`](crate::PropKind::Map)
//! or an [`array`](crate::PropKind::Array). They reflect the [`view`](crate::AttributeView) of the
//! [`Prop`](crate::Prop) (which includes its descendants) and can be used for
//! intra-[`SchemaVariant`](crate::SchemaVariant) connections.
//!
//! ## What are explicit [`InternalProviders`](InternalProvider)?
//!
//! Explicit [`InternalProviders`](InternalProvider) _consume_ values from external
//! [`SchemaVariants`](crate::SchemaVariant), pass them through a transformation [`function`](crate::Func)
//! (usually `si:identity`), and then _expose_ the resulting value within the
//! [`SchemaVariant`](crate::SchemaVariant) that it belongs to.
//!
//! One way to think of explicit [`InternalProviders`](InternalProvider) is as "inverted"
//! [`ExternalProviders`](crate::ExternalProvider). [`ExternalProviders`](crate::ExternalProvider)
//! _consume_ values from within the [`SchemaVariant`](crate::SchemaVariant) that they belong to,
//! pass them through a transformation [`function`](crate::Func) (usually `si:identity`), and then
//! _expose_ the resulting value to external [`SchemaVariants`](crate::SchemaVariant).
//!
//! ## Why the labels "implicit" and "explicit"?
//!
//! The labels originate from the direct and indirect actions of how they are created.
//!
//! [`InternalProviders`](InternalProvider) that _internally consume_ are "implicitly" created when
//! assembling a [`Prop`](crate::Prop) tree for a [`SchemaVariant`](crate::SchemaVariant). They are
//! not "explicitly" created since you get them automatically when
//! [`finalizing`](crate::SchemaVariant::finalize()) a [`SchemaVariant`](crate::SchemaVariant).
//! Conversely, [`InternalProviders`](InternalProvider) for external consumption are "explicitly"
//! created alongside [`Sockets`](crate::Socket) for a [`SchemaVariant`](crate::SchemaVariant).
//!
//! ## Why do implicit [`InternalProviders`](InternalProvider) exist? Can we not just use the values for the [`Props`](crate::Prop) themselves?
//!
//! This was touched on a bit in the "implicit" section, but let's expand on it.
//!
//! [`AttributeValues`](crate::AttributeValue) whose least specific field is a [`Prop`](crate::Prop)
//! in a [`SchemaVariant`](crate::SchemaVariant) contain the value for _solely_ the [`Prop`](crate::Prop)
//! itself. If the [`Prop`](crate::Prop) is an [`object`](crate::PropKind::Object), then you'll likely
//! want to show the value for that [`Prop`](crate::Prop) and its child [`Props`](crate::Prop).
//!
//! ```json
//! {
//!   "data": {
//!     "name": "canoe",
//!     "region": "us-poop-1"
//!   }
//! }
//! ```
//!
//! In the above case, the "data" [`object`](crate::PropKind::Object) [`Prop`](crate::Prop) has two
//! child [`Props`](crate::Prop) of kind [`string`](crate::PropKind::String). If we want to use
//! this entire [`view`](crate::AttributeView), we need an [`AttributeValue`](crate::AttributeValue)
//! for it. What [`AttributeValue`](crate::AttributeValue) contains the view? The
//! [`AttributeValue`](crate::AttributeValue) whose least specific field is the implicit
//! [`InternalProvider`] for the "data" [`Prop`](crate::Prop) (which lives in a
//! [`SchemaVariant`](crate::SchemaVariant)).
//!
//! In addition to the two different [`AttributeValues`](crate::AttributeValue), having implicit
//! [`InternalProviders`](Self) help minimize the number of things that
//! [`AttributePrototypeArguments`](crate::AttributePrototypeArgument) can reference. Need to use
//! a section of the [`Prop`](crate::Prop) tree for a [`SchemaVariant`](crate::SchemaVariant)? No
//! problem, just specify once [`InternalProviderId`](InternalProvider).
//!
//! This design also lets us cache the view of a [`Prop`](crate::Prop) and its children rather
//! than directly observing the real time values frequently.

use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::backend::identity::FuncBackendIdentityArgs;
use crate::func::binding::{FuncBindingError, FuncBindingId};
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::schema::variant::SchemaVariantError;
use crate::socket::{Socket, SocketArity, SocketEdgeKind, SocketError, SocketId, SocketKind};
use crate::standard_model::object_option_from_row_option;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    AttributeContextBuilderError, AttributePrototype, AttributePrototypeError,
    AttributePrototypeId, AttributeReadContext, AttributeValueError, AttributeView, DiagramKind,
    FuncId, HistoryEventError, Prop, PropError, SchemaVariant, StandardModel, StandardModelError,
    Timestamp, Visibility, WriteTenancy,
};
use crate::{
    standard_model_has_many, AttributeContext, AttributeContextError, AttributeValue, DalContext,
    Func, FuncBinding, PropId, SchemaId, SchemaVariantId,
};
use crate::{Component, ComponentId};

const FIND_EXPLICIT_FOR_SCHEMA_VARIANT_AND_NAME: &str =
    include_str!("../queries/internal_provider_find_explicit_for_schema_variant_and_name.sql");
const FIND_FOR_PROP: &str = include_str!("../queries/internal_provider_find_for_prop.sql");
const FIND_EXPLICIT_FOR_SOCKET: &str =
    include_str!("../queries/internal_provider_find_explicit_for_socket.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/internal_provider_list_for_schema_variant.sql");
const LIST_FOR_ATTRIBUTE_PROTOTYPE: &str =
    include_str!("../queries/internal_provider_list_for_attribute_prototype.sql");
const LIST_FOR_INPUT_SOCKETS: &str =
    include_str!("../queries/internal_provider_list_for_input_sockets_for_all_schema_variants.sql");

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
    #[error("component error: {0}")]
    Component(String),
    #[error("component not found by id: {0}")]
    ComponentNotFound(ComponentId),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(String),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("attribute prototype not found for id: {0}")]
    AttributePrototypeNotFound(AttributePrototypeId),
    #[error("could not find attribute value for attribute context: {0:?}")]
    AttributeValueNotFoundForContext(AttributeContext),
    #[error("unexpected: attribute prototype field is empty")]
    EmptyAttributePrototype,
    #[error("func not found for id: {0}")]
    FuncNotFound(FuncId),
    #[error("not allowed to perform implicit emit as an explicit internal provider")]
    ImplicitEmitForExplicitProviderNotAllowed,
    #[error("missing func")]
    MissingFunc(String),
    #[error("provided attribute context does not specify a PropId (required for implicit emit)")]
    MissingPropForImplicitEmit,
    #[error("not found for id: {0}")]
    NotFound(InternalProviderId),
    #[error("internal provider not found for prop id: {0}")]
    NotFoundForProp(PropId),
    #[error("prop not found for id: {0}")]
    PropNotFound(PropId),
    #[error("root prop not found for schema variant: {0}")]
    RootPropNotFound(SchemaVariantId),
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
/// If this provider _specifies_ a [`PropId`](crate::Prop), it provider can only consume data from
/// within its own [`SchemaVariant`](crate::SchemaVariant). Internally-consuming
/// [`InternalProviders`](Self) are called "implicit" [`InternalProviders`](Self).
///
/// If this provider _does not_ specify a [`PropId`](crate::Prop), it can only consume data from
/// other [`SchemaVariants`](crate::SchemaVariant). Externally-consuming [`InternalProviders`](Self)
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
    /// unset if [`Self`] is "explicit". If [`Self`] is "implicit", this will always be a "set" id.
    prop_id: PropId,
    /// Indicates which [`SchemaVariant`](crate::SchemaVariant) this provider belongs to.
    schema_variant_id: SchemaVariantId,
    /// Indicates which transformation function should be used for "emit".
    attribute_prototype_id: Option<AttributePrototypeId>,

    /// Name for [`Self`] that can be used for identification.
    name: String,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    outbound_type_definition: Option<String>,
}

impl InternalProvider {
    #[tracing::instrument(skip(ctx))]
    pub async fn new_implicit(
        ctx: &DalContext,
        prop_id: PropId,
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
                "SELECT object FROM internal_provider_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &prop_id,
                    &schema_variant_id,
                    &name,
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
            FuncBinding::create_and_execute(
                ctx,
                serde_json::json![{ "identity": null }],
                *identity_func.id(),
            )
            .await?;

        // The "base" AttributeContext of anything we create should be as un-specific as possible,
        // and for an InternalProvider that is having only the InternalProviderId set.
        let context = AttributeContext::builder()
            .set_internal_provider_id(*internal_provider.id())
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

    /// This function will also create an _input_ [`Socket`](crate::Socket).
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx, name))]
    pub async fn new_explicit_with_socket(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: impl AsRef<str>,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
        arity: SocketArity,
        diagram_kind: DiagramKind,
    ) -> InternalProviderResult<(Self, Socket)> {
        let name = name.as_ref();
        let prop_id = PropId::NONE;

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM internal_provider_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &prop_id,
                    &schema_variant_id,
                    &name,
                    &Option::<String>::None,
                    &Option::<String>::None,
                ],
            )
            .await?;

        let mut explicit_internal_provider: InternalProvider =
            standard_model::finish_create_from_row(ctx, row).await?;

        // The "base" AttributeContext of anything we create should be as un-specific as possible,
        // and for an InternalProvider that is having only the InternalProviderId set.
        let _base_attribute_context = AttributeContext::builder()
            .set_internal_provider_id(explicit_internal_provider.id)
            .to_context()?;

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

        let socket = Socket::new(
            ctx,
            name,
            SocketKind::Provider,
            &SocketEdgeKind::ConfigurationInput,
            &arity,
            &diagram_kind,
        )
        .await?;
        socket
            .set_internal_provider(ctx, explicit_internal_provider.id())
            .await?;

        let variant = SchemaVariant::get_by_id(ctx, explicit_internal_provider.schema_variant_id())
            .await?
            .ok_or_else(|| {
                InternalProviderError::SchemaVariant(
                    SchemaVariantError::NotFound(*explicit_internal_provider.schema_variant_id())
                        .to_string(),
                )
            })?;
        variant
            .add_socket(ctx, socket.id())
            .await
            .map_err(|err| InternalProviderError::SchemaVariant(err.to_string()))?;

        Ok((explicit_internal_provider, socket))
    }

    // Immutable fields.
    standard_model_accessor_ro!(prop_id, PropId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

    // Mutable fields.
    standard_model_accessor!(
        attribute_prototype_id,
        Option<Pk(AttributePrototypeId)>,
        InternalProviderResult
    );
    standard_model_accessor!(name, String, InternalProviderResult);
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

    // This is a 1-1 relationship, so the Vec<Socket> should be 1
    standard_model_has_many!(
        lookup_fn: sockets,
        table: "socket_belongs_to_internal_provider",
        model_table: "sockets",
        returns: Socket,
        result: InternalProviderResult,
    );

    /// If the [`PropId`](crate::Prop) field is not unset, then [`Self`] is an internal consumer.
    pub fn is_internal_consumer(&self) -> bool {
        self.prop_id != PropId::NONE
    }

    /// Consume with a provided [`AttributeContext`](crate::AttributeContext) and return the
    /// resulting [`AttributeValue`](crate::AttributeValue).
    ///
    /// Requirements for the provided [`AttributeContext`](crate::AttributeContext):
    /// - The least specific field be a [`PropId`](crate::Prop)
    /// - If the [`SchemaId`](crate::Schema) is set, it must match the corresponding field on
    ///   [`Self`]
    /// - If the [`SchemaVariantId`](crate::SchemaVariant) is set, it must match the corresponding
    ///   field on [`Self`]
    pub async fn implicit_emit(
        &self,
        ctx: &DalContext,
        target_attribute_value: &mut AttributeValue,
    ) -> InternalProviderResult<()> {
        if !self.is_internal_consumer() {
            return Err(InternalProviderError::ImplicitEmitForExplicitProviderNotAllowed);
        }

        // Get the func from our attribute prototype.
        let attribute_prototype_id = self
            .attribute_prototype_id
            .ok_or(InternalProviderError::EmptyAttributePrototype)?;
        let attribute_prototype = AttributePrototype::get_by_id(ctx, &attribute_prototype_id)
            .await?
            .ok_or(InternalProviderError::AttributePrototypeNotFound(
                attribute_prototype_id,
            ))?;
        let func_id = attribute_prototype.func_id();
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(InternalProviderError::FuncNotFound(func_id))?;

        // Generate the AttributeContext that we should be sourcing our argument from.
        let consume_attribute_context =
            AttributeContextBuilder::from(target_attribute_value.context)
                .unset_internal_provider_id()
                .unset_external_provider_id()
                .set_prop_id(self.prop_id)
                .to_context()?;

        let source_attribute_value =
            AttributeValue::find_for_context(ctx, consume_attribute_context.into())
                .await?
                .ok_or(InternalProviderError::AttributeValueNotFoundForContext(
                    consume_attribute_context,
                ))?;
        let found_attribute_view_context = AttributeReadContext {
            prop_id: None,
            ..AttributeReadContext::from(consume_attribute_context)
        };

        let found_attribute_view = AttributeView::new(
            ctx,
            found_attribute_view_context,
            Some(*source_attribute_value.id()),
        )
        .await?;
        let (func_binding, func_binding_return_value) = FuncBinding::create_and_execute(
            ctx,
            serde_json::to_value(FuncBackendIdentityArgs {
                identity: found_attribute_view.value().clone(),
            })?,
            *func.id(),
        )
        .await?;

        target_attribute_value
            .set_func_binding_id(ctx, *func_binding.id())
            .await?;
        target_attribute_value
            .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
            .await?;

        if target_attribute_value.context.component_id().is_some() && self.prop_id().is_some() {
            let provider_prop = Prop::get_by_id(ctx, self.prop_id())
                .await?
                .ok_or_else(|| InternalProviderError::PropNotFound(*self.prop_id()))?;

            // NOTE(jhelwig): This whole block will go away once Qualifications/Validations become part of the Prop tree.
            //
            // The Root Prop won't have a parent Prop.
            if provider_prop.parent_prop(ctx).await?.is_none() {
                let component =
                    Component::get_by_id(ctx, &target_attribute_value.context.component_id())
                        .await?
                        .ok_or_else(|| {
                            InternalProviderError::ComponentNotFound(
                                target_attribute_value.context.component_id(),
                            )
                        })?;
                component
                    .check_validations(ctx)
                    .await
                    .map_err(|e| InternalProviderError::Component(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_schema_variant(
        ctx: &DalContext,
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

    /// Find [`Self`] with a provided name, which is not only the name of [`Self`], but also of the
    /// associated _input_ [`Socket`](crate::Socket).
    #[instrument(skip_all)]
    pub async fn find_explicit_for_schema_variant_and_name(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: impl AsRef<str>,
    ) -> InternalProviderResult<Option<Self>> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_EXPLICIT_FOR_SCHEMA_VARIANT_AND_NAME,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &schema_variant_id,
                    &name,
                ],
            )
            .await?;
        Ok(object_option_from_row_option(row)?)
    }

    /// Find [`Self`] with a provided [`SocketId`](crate::Socket).
    #[instrument(skip_all)]
    pub async fn find_explicit_for_socket(
        ctx: &DalContext,
        socket_id: SocketId,
    ) -> InternalProviderResult<Option<Self>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_EXPLICIT_FOR_SOCKET,
                &[ctx.read_tenancy(), ctx.visibility(), &socket_id],
            )
            .await?;
        Ok(object_option_from_row_option(row)?)
    }

    /// Find all [`Self`] for a given [`AttributePrototypeId`](crate::AttributePrototype).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_attribute_prototype(
        ctx: &DalContext,
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

    /// Find all [`Self`] which are also input sockets.
    pub async fn list_for_input_sockets(ctx: &DalContext) -> InternalProviderResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_INPUT_SOCKETS,
                &[ctx.read_tenancy(), ctx.visibility()],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Returns an [`AttributeContext`](crate::AttributeContext) corresponding to our id.
    pub fn attribute_context(&self) -> InternalProviderResult<AttributeContext> {
        Ok(AttributeContext::builder()
            .set_internal_provider_id(self.id)
            .to_context()?)
    }

    /// Finds [`Self`] for a given [`PropId`](crate::Prop). This will only work for
    /// implicit [`InternalProviders`](Self).
    pub async fn find_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> InternalProviderResult<Option<Self>> {
        let row = ctx
            .pg_txn()
            .query_opt(
                FIND_FOR_PROP,
                &[ctx.read_tenancy(), ctx.visibility(), &prop_id],
            )
            .await?;
        Ok(object_option_from_row_option(row)?)
    }
}
