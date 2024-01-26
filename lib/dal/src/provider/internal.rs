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
//! [`SchemaVariants`](crate::SchemaVariant), pass them through a transformation [`Func`](crate::Func)
//! (usually the identity [`Func`](crate::Func)), and then _expose_ the resulting value within the
//! [`SchemaVariant`](crate::SchemaVariant) that it belongs to.
//!
//! One way to think of explicit [`InternalProviders`](InternalProvider) is as "inverted"
//! [`ExternalProviders`](crate::ExternalProvider). [`ExternalProviders`](crate::ExternalProvider)
//! _consume_ values from within the [`SchemaVariant`](crate::SchemaVariant) that they belong to,
//! pass them through a transformation [`Func`](crate::Func) (usually the identity [`Func`](crate::Func)), and then
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

use content_store::{ContentHash, Store};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::FuncError;
use crate::provider::{ProviderArity, ProviderKind};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, AttributePrototype, AttributePrototypeId, AttributeValueId, DalContext, FuncId, PropId,
    SchemaVariantId, Timestamp, TransactionsError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InternalProviderError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("missing attribute prototype for explicit internal provider: {0}")]
    MissingAttributePrototypeExplicit(InternalProviderId),
    #[error("missing attribute value for explicit internal provider: {0}")]
    MissingAttributeValueExplicit(InternalProviderId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("Prop {0} is missing an internal provider")]
    PropMissingInternalProvider(PropId),
    #[error("An internal provider for prop {0} already exists")]
    ProviderAlreadyExists(PropId),
    #[error("store error: {0}")]
    Store(#[from] content_store::StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type InternalProviderResult<T> = Result<T, InternalProviderError>;

pk!(InternalProviderId);

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
    id: InternalProviderId,
    #[serde(flatten)]
    timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    name: String,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    outbound_type_definition: Option<String>,
    arity: ProviderArity,
    kind: ProviderKind,
    required: bool,
    ui_hidden: bool,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum InternalProviderContent {
    V1(InternalProviderContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InternalProviderContentV1 {
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    pub name: String,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    pub inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    pub outbound_type_definition: Option<String>,
    pub arity: ProviderArity,
    pub kind: ProviderKind,
    pub required: bool,
    pub ui_hidden: bool,
}

impl InternalProvider {
    pub async fn get_by_id(
        ctx: &DalContext,
        id: InternalProviderId,
    ) -> InternalProviderResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let node_weight = workspace_snapshot.get_node_weight_by_id(id)?;

        Self::get_from_node_weight(ctx, node_weight).await
    }

    pub fn assemble(id: InternalProviderId, inner: InternalProviderContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            inbound_type_definition: inner.inbound_type_definition,
            outbound_type_definition: inner.outbound_type_definition,
            arity: inner.arity,
            kind: inner.kind,
            required: inner.required,
            ui_hidden: inner.ui_hidden,
        }
    }

    pub fn id(&self) -> InternalProviderId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> ProviderArity {
        self.arity
    }

    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }

    pub fn required(&self) -> bool {
        self.required
    }

    async fn get_from_node_weight(
        ctx: &DalContext,
        node_weight: &NodeWeight,
    ) -> InternalProviderResult<Self> {
        let content: InternalProviderContent = ctx
            .content_store()
            .try_lock()?
            .get(&node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                node_weight.id(),
            ))?;

        let InternalProviderContent::V1(inner) = content;

        Ok(Self::assemble(node_weight.id().into(), inner))
    }

    pub async fn add_prototype_edge(
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
        attribute_prototype_id: AttributePrototypeId,
        key: &Option<String>,
    ) -> InternalProviderResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
        workspace_snapshot.add_edge(
            internal_provider_id,
            EdgeWeight::new(
                ctx.change_set_pointer()?,
                EdgeWeightKind::Prototype(key.to_owned()),
            )?,
            attribute_prototype_id,
        )?;

        Ok(())
    }

    pub async fn find_explicit_with_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> InternalProviderResult<Option<Self>> {
        let name = name.as_ref();

        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        for provider_idx in workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Provider,
        )? {
            let node_weight = workspace_snapshot.get_node_weight(provider_idx)?;
            if let NodeWeight::Content(content_inner) = node_weight {
                if ContentAddressDiscriminants::InternalProvider
                    == content_inner.content_address().into()
                {
                    let ip = Self::get_from_node_weight(ctx, node_weight).await?;
                    if ip.name() == name {
                        return Ok(Some(ip));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn new_explicit(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: impl Into<String>,
        func_id: FuncId,
        arity: ProviderArity,
        kind: ProviderKind,
        // todo: connection_annotation
    ) -> InternalProviderResult<Self> {
        info!("creating explicit internal provider");
        let name = name.into();
        let content = InternalProviderContentV1 {
            timestamp: Timestamp::now(),
            name: name.clone(),
            inbound_type_definition: None,
            outbound_type_definition: None,
            arity,
            kind,
            required: false,
            ui_hidden: false,
        };
        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&InternalProviderContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            let node_weight =
                NodeWeight::new_content(change_set, id, ContentAddress::InternalProvider(hash))?;
            let _node_index = workspace_snapshot.add_node(node_weight)?;
            workspace_snapshot.add_edge(
                schema_variant_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Provider)?,
                id,
            )?;
        }

        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            workspace_snapshot.add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::Prototype(None))?,
                attribute_prototype.id(),
            )?;
        }

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn list_ids_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InternalProviderResult<Vec<InternalProviderId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let node_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Provider,
        )?;

        let mut result = vec![];
        for node_index in node_indices {
            let node_weight = workspace_snapshot.get_node_weight(node_index)?;
            if node_weight
                .get_option_content_node_weight_of_kind(
                    ContentAddressDiscriminants::InternalProvider,
                )
                .is_some()
            {
                result.push(node_weight.id().into());
            }
        }

        Ok(result)
    }

    pub async fn list(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InternalProviderResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let node_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Provider,
        )?;

        let mut content_hashes = Vec::new();
        let mut node_weights = Vec::new();
        for node_index in node_indices {
            let node_weight = workspace_snapshot.get_node_weight(node_index)?;
            if let Some(content_node_weight) = node_weight.get_option_content_node_weight_of_kind(
                ContentAddressDiscriminants::InternalProvider,
            ) {
                content_hashes.push(content_node_weight.content_hash());
                node_weights.push(content_node_weight);
            }
        }

        let content_map: HashMap<ContentHash, InternalProviderContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(content_hashes.as_slice())
            .await?;

        let mut internal_providers = Vec::new();
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let InternalProviderContent::V1(inner) = content;

                    internal_providers
                        .push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(internal_providers)
    }

    pub async fn attribute_values_for_internal_provider_id(
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
    ) -> InternalProviderResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let av_sources = workspace_snapshot.incoming_sources_for_edge_weight_kind(
            internal_provider_id,
            EdgeWeightKindDiscriminants::Provider,
        )?;
        for av_source_idx in av_sources {
            if let NodeWeight::AttributeValue(av_node_weight) =
                workspace_snapshot.get_node_weight(av_source_idx)?
            {
                result.push(av_node_weight.id().into());
            }
        }

        Ok(result)
    }
}

// impl InternalProvider {
//     #[tracing::instrument(skip(ctx))]
//     pub async fn new_implicit(
//         ctx: &DalContext,
//         prop_id: PropId,
//         schema_variant_id: SchemaVariantId,
//     ) -> InternalProviderResult<Self> {
//         // Use the prop name for the implicit internal provider name. We need an owned string that
//         // we then borrow for the query.
//         let prop = Prop::get_by_id(ctx, &prop_id)
//             .await?
//             .ok_or(InternalProviderError::PropNotFound(prop_id))?;
//         let name = prop.name().to_string();

//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM internal_provider_create_v1($1, $2, $3, $4, $5, $6, $7)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &prop_id,
//                     &schema_variant_id,
//                     &name,
//                     &Option::<String>::None,
//                     &Option::<String>::None,
//                 ],
//             )
//             .await?;
//         let mut internal_provider: InternalProvider =
//             standard_model::finish_create_from_row(ctx, row).await?;

//         let (identity_func, identity_func_binding, identity_func_binding_return_value) =
//             Func::identity_with_binding_and_return_value(ctx).await?;

//         // The "base" AttributeContext of anything we create should be as un-specific as possible,
//         // and for an InternalProvider that is having only the InternalProviderId set.
//         let context = AttributeContext::builder()
//             .set_internal_provider_id(*internal_provider.id())
//             .to_context()?;

//         // Key and parent are unneeded because the provider exists not strictly as part of the
//         // schema values _and_ because implicit internal providers cannot be created for descendants
//         // of maps and arrays.
//         let attribute_prototype = AttributePrototype::new(
//             ctx,
//             *identity_func.id(),
//             *identity_func_binding.id(),
//             *identity_func_binding_return_value.id(),
//             context,
//             None,
//             None,
//         )
//         .await?;

//         internal_provider
//             .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
//             .await?;
//         Ok(internal_provider)
//     }

//     /// This function will also create an _input_ [`Socket`](crate::Socket).
//     #[allow(clippy::too_many_arguments)]
//     #[tracing::instrument(skip(ctx, name))]
//     pub async fn new_explicit_with_socket(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         name: impl AsRef<str>,
//         func_id: FuncId,
//         func_binding_id: FuncBindingId,
//         func_binding_return_value_id: FuncBindingReturnValueId,
//         arity: SocketArity,
//         frame_socket: bool,
//     ) -> InternalProviderResult<(Self, Socket)> {
//         let name = name.as_ref();
//         let prop_id = PropId::NONE;

//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM internal_provider_create_v1($1, $2, $3, $4, $5, $6, $7)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &prop_id,
//                     &schema_variant_id,
//                     &name,
//                     &Option::<String>::None,
//                     &Option::<String>::None,
//                 ],
//             )
//             .await?;

//         let mut explicit_internal_provider: InternalProvider =
//             standard_model::finish_create_from_row(ctx, row).await?;

//         // The "base" AttributeContext of anything we create should be as un-specific as possible,
//         // and for an InternalProvider that is having only the InternalProviderId set.
//         let _base_attribute_context = AttributeContext::builder()
//             .set_internal_provider_id(explicit_internal_provider.id)
//             .to_context()?;

//         let attribute_prototype = AttributePrototype::new(
//             ctx,
//             func_id,
//             func_binding_id,
//             func_binding_return_value_id,
//             explicit_internal_provider.attribute_context()?,
//             None,
//             None,
//         )
//         .await?;
//         explicit_internal_provider
//             .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
//             .await?;

//         let socket = Socket::new(
//             ctx,
//             name,
//             match frame_socket {
//                 true => SocketKind::Frame,
//                 false => SocketKind::Provider,
//             },
//             &SocketEdgeKind::ConfigurationInput,
//             &arity,
//             &DiagramKind::Configuration,
//             Some(schema_variant_id),
//         )
//         .await?;
//         socket
//             .set_internal_provider(ctx, explicit_internal_provider.id())
//             .await?;

//         Ok((explicit_internal_provider, socket))
//     }

//     // Immutable fields.
//     standard_model_accessor_ro!(prop_id, PropId);
//     standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

//     // Mutable fields.
//     standard_model_accessor!(
//         attribute_prototype_id,
//         Option<Pk(AttributePrototypeId)>,
//         InternalProviderResult
//     );
//     standard_model_accessor!(name, String, InternalProviderResult);
//     standard_model_accessor!(
//         inbound_type_definition,
//         Option<String>,
//         InternalProviderResult
//     );
//     standard_model_accessor!(
//         outbound_type_definition,
//         Option<String>,
//         InternalProviderResult
//     );

//     // This is a 1-1 relationship, so the Vec<Socket> should be 1
//     standard_model_has_many!(
//         lookup_fn: sockets,
//         table: "socket_belongs_to_internal_provider",
//         model_table: "sockets",
//         returns: Socket,
//         result: InternalProviderResult,
//     );

//     /// If the [`PropId`](crate::Prop) field is not unset, then [`Self`] is an internal consumer.
//     pub fn is_internal_consumer(&self) -> bool {
//         self.prop_id != PropId::NONE
//     }

//     /// Consume with a provided [`AttributeContext`](crate::AttributeContext) and return the
//     /// resulting [`AttributeValue`](crate::AttributeValue).
//     ///
//     /// Requirements for the provided [`AttributeContext`](crate::AttributeContext):
//     /// - The least specific field be a [`PropId`](crate::Prop)
//     /// - If the [`SchemaId`](crate::Schema) is set, it must match the corresponding field on
//     ///   [`Self`]
//     /// - If the [`SchemaVariantId`](crate::SchemaVariant) is set, it must match the corresponding
//     ///   field on [`Self`]
//     pub async fn implicit_emit(
//         &self,
//         ctx: &DalContext,
//         target_attribute_value: &mut AttributeValue,
//     ) -> InternalProviderResult<()> {
//         if !self.is_internal_consumer() {
//             return Err(InternalProviderError::ImplicitEmitForExplicitProviderNotAllowed);
//         }

//         // Get the func from our attribute prototype.
//         let attribute_prototype_id = self
//             .attribute_prototype_id
//             .ok_or(InternalProviderError::EmptyAttributePrototype)?;
//         let attribute_prototype = AttributePrototype::get_by_id(ctx, &attribute_prototype_id)
//             .await?
//             .ok_or(InternalProviderError::AttributePrototypeNotFound(
//                 attribute_prototype_id,
//             ))?;
//         let func_id = attribute_prototype.func_id();
//         let func = Func::get_by_id(ctx, &func_id)
//             .await?
//             .ok_or(InternalProviderError::FuncNotFound(func_id))?;

//         // Generate the AttributeContext that we should be sourcing our argument from.
//         let consume_attribute_context =
//             AttributeContextBuilder::from(target_attribute_value.context)
//                 .unset_internal_provider_id()
//                 .unset_external_provider_id()
//                 .set_prop_id(self.prop_id)
//                 .to_context()?;

//         let source_attribute_value =
//             AttributeValue::find_for_context(ctx, consume_attribute_context.into())
//                 .await?
//                 .ok_or(InternalProviderError::AttributeValueNotFoundForContext(
//                     consume_attribute_context,
//                 ))?;
//         let found_attribute_view_context = AttributeReadContext {
//             prop_id: None,
//             ..AttributeReadContext::from(consume_attribute_context)
//         };

// let found_attribute_view = AttributeView::new(
//     ctx,
//     found_attribute_view_context,
//     Some(*source_attribute_value.id()),
// )
// .await?;

// let (func_binding, func_binding_return_value) = FuncBinding::create_and_execute(
//     ctx,
//     serde_json::to_value(FuncBackendIdentityArgs {
//         identity: Some(found_attribute_view.value().clone()),
//     })?,
//     *func.id(),
//     vec![],
// )
// .await?;

//         target_attribute_value
//             .set_func_binding_id(ctx, *func_binding.id())
//             .await?;
//         target_attribute_value
//             .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
//             .await?;

//         if target_attribute_value.context.component_id().is_some() && self.prop_id().is_some() {
//             let provider_prop = Prop::get_by_id(ctx, self.prop_id())
//                 .await?
//                 .ok_or_else(|| InternalProviderError::PropNotFound(*self.prop_id()))?;

//             // NOTE(jhelwig): This whole block will go away once Qualifications/Validations become part of the Prop tree.
//             //
//             // The Root Prop won't have a parent Prop.
//             if provider_prop.parent_prop(ctx).await?.is_none() {
//                 let ctx_deletion = &ctx.clone_with_delete_visibility();
//                 let component = Component::get_by_id(
//                     ctx_deletion,
//                     &target_attribute_value.context.component_id(),
//                 )
//                 .await?
//                 .ok_or_else(|| {
//                     InternalProviderError::ComponentNotFound(
//                         target_attribute_value.context.component_id(),
//                     )
//                 })?;
//                 component
//                     .check_validations(ctx)
//                     .await
//                     .map_err(|e| InternalProviderError::Component(e.to_string()))?;
//             }
//         }

//         Ok(())
//     }

//     /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> InternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_SCHEMA_VARIANT,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_explicit_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> InternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_EXPLICIT_FOR_SCHEMA_VARIANT,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find [`Self`] with a provided name, which is not only the name of [`Self`], but also of the
//     /// associated _input_ [`Socket`](crate::Socket).
//     #[instrument(skip_all)]
//     pub async fn find_explicit_for_schema_variant_and_name(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         name: impl AsRef<str>,
//     ) -> InternalProviderResult<Option<Self>> {
//         let name = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_EXPLICIT_FOR_SCHEMA_VARIANT_AND_NAME,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id, &name],
//             )
//             .await?;
//         Ok(object_option_from_row_option(row)?)
//     }

//     /// Find [`Self`] with a provided [`SocketId`](crate::Socket).
//     #[instrument(skip_all)]
//     pub async fn find_explicit_for_socket(
//         ctx: &DalContext,
//         socket_id: SocketId,
//     ) -> InternalProviderResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_EXPLICIT_FOR_SOCKET,
//                 &[ctx.tenancy(), ctx.visibility(), &socket_id],
//             )
//             .await?;
//         Ok(object_option_from_row_option(row)?)
//     }

//     /// Find all [`Self`] for a given [`AttributePrototypeId`](crate::AttributePrototype).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_for_attribute_prototype(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//     ) -> InternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_ATTRIBUTE_PROTOTYPE,
//                 &[ctx.tenancy(), ctx.visibility(), &attribute_prototype_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find all [`Self`] which are also input sockets.
//     pub async fn list_for_input_sockets(
//         ctx: &DalContext,
//         schema_variant_id: Option<SchemaVariantId>,
//     ) -> InternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_INPUT_SOCKETS,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Returns an [`AttributeContext`](crate::AttributeContext) corresponding to our id.
//     pub fn attribute_context(&self) -> InternalProviderResult<AttributeContext> {
//         Ok(AttributeContext::builder()
//             .set_internal_provider_id(self.id)
//             .to_context()?)
//     }

// /// Finds [`Self`] for a given [`PropId`](crate::Prop). This will only work for
// /// implicit [`InternalProviders`](Self).
// pub async fn find_for_prop(
//     ctx: &DalContext,
//     prop_id: PropId,
// ) -> InternalProviderResult<Option<Self>> {
//     let row = ctx
//         .txns()
//         .await?
//         .pg()
//         .query_opt(FIND_FOR_PROP, &[ctx.tenancy(), ctx.visibility(), &prop_id])
//         .await?;
//     Ok(object_option_from_row_option(row)?)
// }

// #[tracing::instrument(skip(ctx))]
// pub async fn by_socket(ctx: &DalContext) -> InternalProviderResult<HashMap<SocketId, Self>> {
//     let rows = ctx
//         .txns()
//         .await?
//         .pg()
//         .query(BY_SOCKET, &[ctx.tenancy(), ctx.visibility()])
//         .await?;

//     let mut objects: HashMap<SocketId, Self> = HashMap::new();
//     for row in rows.into_iter() {
//         let id: SocketId = row.try_get(0)?;

//         let object: serde_json::Value = row.try_get(1)?;
//         let object: Self = serde_json::from_value(object)?;

//         objects.insert(id, object);
//     }

//     Ok(objects.into_iter().collect())
// }
// }
