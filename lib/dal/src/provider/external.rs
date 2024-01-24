use content_store::{ContentHash, Store};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::provider::{ProviderArity, ProviderKind};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{pk, AttributePrototype, DalContext, FuncId, Timestamp, TransactionsError};
use crate::{AttributeValueId, SchemaVariantId};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ExternalProviderError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("store error: {0}")]
    Store(#[from] content_store::StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ExternalProviderResult<T> = Result<T, ExternalProviderError>;

pk!(ExternalProviderId);

/// This provider can only provide data to external [`SchemaVariants`](crate::SchemaVariant). It can
/// only consume data within its own [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ExternalProvider {
    id: ExternalProviderId,
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    name: String,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
    arity: ProviderArity,
    kind: ProviderKind,
    required: bool,
    ui_hidden: bool,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ExternalProviderContent {
    V1(ExternalProviderContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExternalProviderContentV1 {
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    pub name: String,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    pub type_definition: Option<String>,
    pub arity: ProviderArity,
    pub kind: ProviderKind,
    pub required: bool,
    pub ui_hidden: bool,
}

impl ExternalProvider {
    pub fn assemble(id: ExternalProviderId, inner: ExternalProviderContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            type_definition: inner.type_definition,
            arity: inner.arity,
            kind: inner.kind,
            ui_hidden: inner.ui_hidden,
            required: inner.required,
        }
    }

    pub fn id(&self) -> ExternalProviderId {
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

    pub async fn new(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: impl Into<String>,
        type_definition: Option<String>,
        func_id: FuncId,
        arity: ProviderArity,
        kind: ProviderKind,
        // todo: connection_annotation
    ) -> ExternalProviderResult<Self> {
        let name = name.into();
        let content = ExternalProviderContentV1 {
            timestamp: Timestamp::now(),
            name: name.clone(),
            type_definition,
            arity,
            kind,
            required: false,
            ui_hidden: false,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ExternalProviderContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ExternalProvider(hash))?;
        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
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

    pub async fn attribute_values_for_external_provider_id(
        ctx: &DalContext,
        external_provider_id: ExternalProviderId,
    ) -> ExternalProviderResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let av_sources = workspace_snapshot.incoming_sources_for_edge_weight_kind(
            external_provider_id,
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

    pub async fn list(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ExternalProviderResult<Vec<Self>> {
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
                ContentAddressDiscriminants::ExternalProvider,
            ) {
                content_hashes.push(content_node_weight.content_hash());
                node_weights.push(content_node_weight);
            }
        }

        let content_map: HashMap<ContentHash, ExternalProviderContent> = ctx
            .content_store()
            .lock()
            .await
            .get_bulk(content_hashes.as_slice())
            .await?;

        let mut external_providers = Vec::new();
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let ExternalProviderContent::V1(inner) = content;

                    external_providers
                        .push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(external_providers)
    }
}

// impl ExternalProvider {
//     /// This function will also create an _output_ [`Socket`](crate::Socket).
//     #[allow(clippy::too_many_arguments)]
//     #[tracing::instrument(skip(ctx, name))]
//     pub async fn new_with_socket(
//         ctx: &DalContext,
//         schema_id: SchemaId,
//         schema_variant_id: SchemaVariantId,
//         name: impl AsRef<str>,
//         type_definition: Option<String>,
//         func_id: FuncId,
//         func_binding_id: FuncBindingId,
//         func_binding_return_value_id: FuncBindingReturnValueId,
//         arity: SocketArity,
//         frame_socket: bool,
//     ) -> ExternalProviderResult<(Self, Socket)> {
//         let name = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM external_provider_create_v1($1, $2, $3, $4, $5, $6)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &schema_id,
//                     &schema_variant_id,
//                     &name,
//                     &type_definition,
//                 ],
//             )
//             .await?;

//         let mut external_provider: ExternalProvider =
//             standard_model::finish_create_from_row(ctx, row).await?;

//         let attribute_context = AttributeContext::builder()
//             .set_external_provider_id(external_provider.id)
//             .to_context()?;
//         let attribute_prototype = AttributePrototype::new(
//             ctx,
//             func_id,
//             func_binding_id,
//             func_binding_return_value_id,
//             attribute_context,
//             None,
//             None,
//         )
//         .await?;
//         external_provider
//             .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
//             .await?;

//         let socket = Socket::new(
//             ctx,
//             name,
//             match frame_socket {
//                 true => SocketKind::Frame,
//                 false => SocketKind::Provider,
//             },
//             &SocketEdgeKind::ConfigurationOutput,
//             &arity,
//             &DiagramKind::Configuration,
//             Some(schema_variant_id),
//         )
//         .await?;
//         socket
//             .set_external_provider(ctx, external_provider.id())
//             .await?;

//         Ok((external_provider, socket))
//     }

//     // Immutable fields.
//     standard_model_accessor_ro!(schema_id, SchemaId);
//     standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

//     // Mutable fields.
//     standard_model_accessor!(name, String, ExternalProviderResult);
//     standard_model_accessor!(type_definition, Option<String>, ExternalProviderResult);
//     standard_model_accessor!(
//         attribute_prototype_id,
//         Option<Pk(AttributePrototypeId)>,
//         ExternalProviderResult
//     );

//     // This is a 1-1 relationship, so the Vec<Socket> should be 1
//     standard_model_has_many!(
//         lookup_fn: sockets,
//         table: "socket_belongs_to_external_provider",
//         model_table: "sockets",
//         returns: Socket,
//         result: ExternalProviderResult,
//     );

//     /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> ExternalProviderResult<Vec<Self>> {
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

//     /// Find [`Self`] with a provided [`SocketId`](crate::Socket).
//     #[instrument(skip_all)]
//     pub async fn find_for_socket(
//         ctx: &DalContext,
//         socket_id: SocketId,
//     ) -> ExternalProviderResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_FOR_SOCKET,
//                 &[ctx.tenancy(), ctx.visibility(), &socket_id],
//             )
//             .await?;
//         Ok(standard_model::object_option_from_row_option(row)?)
//     }

//     /// Find [`Self`] with a provided name, which is not only the name of [`Self`], but also of the
//     /// associated _output_ [`Socket`](crate::Socket).
//     #[instrument(skip_all)]
//     pub async fn find_for_schema_variant_and_name(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         name: impl AsRef<str>,
//     ) -> ExternalProviderResult<Option<Self>> {
//         let name = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_FOR_SCHEMA_VARIANT_AND_NAME,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id, &name],
//             )
//             .await?;
//         Ok(standard_model::object_option_from_row_option(row)?)
//     }

//     /// Find all [`Self`] for a given [`AttributePrototypeId`](crate::AttributePrototype).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_for_attribute_prototype_with_tail_component_id(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         tail_component_id: ComponentId,
//     ) -> ExternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_ATTRIBUTE_PROTOTYPE_WITH_TAIL_COMPONENT_ID,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_prototype_id,
//                     &tail_component_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

// /// Find all [`Self`] that have
// /// [`AttributePrototypeArguments`](crate::AttributePrototypeArgument) referencing the provided
// /// [`InternalProviderId`](crate::InternalProvider).
// #[tracing::instrument(skip(ctx))]
// pub async fn list_from_internal_provider_use(
//     ctx: &DalContext,
//     internal_provider_id: InternalProviderId,
// ) -> ExternalProviderResult<Vec<Self>> {
//     let rows = ctx
//         .txns()
//         .await?
//         .pg()
//         .query(
//             LIST_FROM_INTERNAL_PROVIDER_USE,
//             &[ctx.tenancy(), ctx.visibility(), &internal_provider_id],
//         )
//         .await?;
//     Ok(standard_model::objects_from_rows(rows)?)
// }

// #[tracing::instrument(skip(ctx))]
// pub async fn by_socket(ctx: &DalContext) -> ExternalProviderResult<HashMap<SocketId, Self>> {
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
//     Ok(objects)
// }
// }
