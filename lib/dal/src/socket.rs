use blake3::hash;
use content_store::{ContentHash, Store};
use object_tree::TarReadError::Hash;
use serde::{Deserialize, Serialize};
use si_pkg::SocketSpecArity;
use std::collections::HashMap;
use std::process::id;
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};
use telemetry::prelude::info;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::provider::external::ExternalProviderContent;
use crate::provider::internal::InternalProviderContent;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    label_list::ToLabelList, pk, socket, DalContext, ExternalProvider, ExternalProviderId,
    InternalProvider, InternalProviderId, SchemaVariantId, StandardModel, Timestamp,
    TransactionsError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SocketError {
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

pub type SocketResult<T> = Result<T, SocketError>;

pk!(SocketId);

// TODO(nick,zack,jacob): this is temporary. Move back to "diagram.rs" later.
#[remain::sorted]
#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DiagramKind {
    /// Represents the collection of [`Components`](crate::Component) and connections between them
    /// within a [`Workspace`](crate::Workspace)
    Configuration,
}

/// The mechanism for setting relationships between [`SchemaVariants`](crate::SchemaVariant) or
/// instantiations of the same [`SchemaVariant`](crate::SchemaVariant).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Socket {
    id: SocketId,
    #[serde(flatten)]
    timestamp: Timestamp,
    name: String,
    human_name: Option<String>,
    kind: SocketKind,
    edge_kind: SocketEdgeKind,
    diagram_kind: DiagramKind,
    arity: SocketArity,
    required: bool,
    ui_hidden: bool,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SocketContent {
    V1(SocketContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SocketContentV1 {
    pub timestamp: Timestamp,
    pub name: String,
    pub human_name: Option<String>,
    pub kind: SocketKind,
    pub edge_kind: SocketEdgeKind,
    pub diagram_kind: DiagramKind,
    pub arity: SocketArity,
    pub required: bool,
    pub ui_hidden: bool,
}

/// Dictates the kind of behavior possible for a [`Socket`](Socket).
#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketKind {
    /// Indicates that this [`Socket`](Socket) is for use with "frames"  _and_ was created
    /// alongside [`provider`](crate::provider).
    Frame,
    /// Indicates that this [`Socket`](Socket) was created alongside a
    /// [`provider`](crate::provider).
    Provider,
    /// Indicates that this [`Socket`](Socket) was _not_ created alongside a
    /// [`provider`](crate::provider).
    Standalone,
}

#[remain::sorted]
#[derive(
    AsRefStr, Clone, Debug, Deserialize, Display, EnumIter, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketArity {
    Many,
    One,
}

impl From<&SocketArity> for SocketSpecArity {
    fn from(value: &SocketArity) -> Self {
        match value {
            SocketArity::One => Self::One,
            SocketArity::Many => Self::Many,
        }
    }
}

impl From<SocketSpecArity> for SocketArity {
    fn from(value: SocketSpecArity) -> Self {
        match value {
            SocketSpecArity::One => Self::One,
            SocketSpecArity::Many => Self::Many,
        }
    }
}

impl ToLabelList for SocketArity {}

/// Dictates the kind of [`Edges`](crate::Edge) that can be created for a [`Socket`](Socket).
#[remain::sorted]
#[derive(
    AsRefStr,
    Copy,
    Clone,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketEdgeKind {
    /// The kind used for [`Sockets`](crate::Socket) created during
    /// [`InternalProvider::new_explicit_with_socket()`].
    ConfigurationInput,
    /// The kind used for [`Sockets`](crate::Socket) created during
    /// [`ExternalProvider::new_with_socket()`].
    ConfigurationOutput,
}

impl ToLabelList for SocketEdgeKind {}

pub enum SocketParent {
    ExplicitInternalProvider(InternalProviderId),
    ExternalProvider(ExternalProviderId),
}

impl Socket {
    pub fn assemble(id: SocketId, inner: SocketContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name.clone(),
            human_name: inner.human_name,
            kind: inner.kind,
            edge_kind: inner.edge_kind,
            diagram_kind: inner.diagram_kind,
            arity: inner.arity,
            required: inner.required,
            ui_hidden: inner.ui_hidden,
        }
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: SocketKind,
        socket_edge_kind: SocketEdgeKind,
        arity: SocketArity,
        diagram_kind: DiagramKind,
        socket_parent: SocketParent,
    ) -> SocketResult<Self> {
        info!("creating socket");
        let content = SocketContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            human_name: None,
            kind,
            edge_kind: socket_edge_kind,
            diagram_kind,
            arity,
            required: false,
            ui_hidden: false,
        };
        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&SocketContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Socket(hash))?;
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let _node_index = workspace_snapshot.add_node(node_weight)?;

        let provider_id: Ulid = match socket_parent {
            SocketParent::ExplicitInternalProvider(explicit_internal_provider_id) => {
                explicit_internal_provider_id.into()
            }
            SocketParent::ExternalProvider(external_provider_id) => external_provider_id.into(),
        };

        workspace_snapshot.add_edge(
            provider_id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            id,
        )?;

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn list_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SocketResult<(Vec<(Self, InternalProvider)>, Vec<(Self, ExternalProvider)>)> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        // Look for all external and explicit internal providers that the schema variant uses.
        let maybe_provider_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Use,
        )?;

        // Collect the external and the explicit internal providers separately. Along the way,
        // collect all the provider ids into one vec.
        let mut external_provider_id_map: HashMap<ExternalProviderId, ContentHash> = HashMap::new();
        let mut external_provider_hashes = Vec::new();
        let mut internal_provider_id_map: HashMap<InternalProviderId, ContentHash> = HashMap::new();
        let mut internal_provider_hashes = Vec::new();
        let mut providers_seen = Vec::new();
        for maybe_provider_index in maybe_provider_indices {
            let node_weight = workspace_snapshot.get_node_weight(maybe_provider_index)?;
            if let NodeWeight::Content(content_node_weight) = node_weight {
                match content_node_weight.content_address() {
                    ContentAddress::InternalProvider(internal_provider_content_hash) => {
                        dbg!("internal provider found");
                        internal_provider_id_map.insert(
                            content_node_weight.id().into(),
                            internal_provider_content_hash,
                        );
                        internal_provider_hashes.push(internal_provider_content_hash);
                        providers_seen.push(content_node_weight.id());
                    }
                    ContentAddress::ExternalProvider(external_provider_content_hash) => {
                        dbg!("external provider found");
                        external_provider_id_map.insert(
                            content_node_weight.id().into(),
                            external_provider_content_hash,
                        );
                        external_provider_hashes.push(external_provider_content_hash);
                        providers_seen.push(content_node_weight.id());
                    }
                    _ => {}
                }
            }
        }

        // Collect all the sockets as well as what provider each socket belongs to.
        let mut socket_hashes = Vec::new();
        let mut provider_to_socket_map: HashMap<Ulid, (SocketId, ContentHash)> = HashMap::new();
        for provider_seen in providers_seen {
            let maybe_output_socket_indices = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    provider_seen,
                    EdgeWeightKindDiscriminants::Use,
                )?;

            for maybe_output_socket_index in maybe_output_socket_indices {
                let node_weight = workspace_snapshot.get_node_weight(maybe_output_socket_index)?;
                if let Ok(socket_node_weight) =
                    node_weight.get_content_node_weight_of_kind(ContentAddressDiscriminants::Socket)
                {
                    socket_hashes.push(socket_node_weight.content_hash());
                    if provider_to_socket_map
                        .insert(
                            provider_seen,
                            (
                                socket_node_weight.id().into(),
                                socket_node_weight.content_hash(),
                            ),
                        )
                        .is_some()
                    {
                        panic!("more than one socket found for the provider");
                    }
                }
            }
        }

        // Grab all the contents in bulk from the content store.
        let external_provider_content_map: HashMap<ContentHash, ExternalProviderContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(external_provider_hashes.as_slice())
            .await?;
        let internal_provider_content_map: HashMap<ContentHash, InternalProviderContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(internal_provider_hashes.as_slice())
            .await?;
        let socket_content_map: HashMap<ContentHash, SocketContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(socket_hashes.as_slice())
            .await?;

        // Collect all input sockets with their providers.
        let mut input_sockets: Vec<(Socket, InternalProvider)> =
            Vec::with_capacity(internal_provider_id_map.keys().len());
        for (internal_provider_id, internal_provider_hash) in internal_provider_id_map {
            let internal_provider_content = internal_provider_content_map
                .get(&internal_provider_hash)
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    internal_provider_id.into(),
                ))?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let InternalProviderContent::V1(internal_provider_content_inner) =
                internal_provider_content;
            let internal_provider = InternalProvider::assemble(
                internal_provider_id,
                internal_provider_content_inner.to_owned(),
            );

            // Now that we have the provider, assemble the socket.
            let (socket_id, socket_hash) = provider_to_socket_map
                .get(&internal_provider_id.into())
                .expect("no socket for provider");
            let socket_content = socket_content_map.get(socket_hash).ok_or(
                WorkspaceSnapshotError::MissingContentFromStore(socket_id.into()),
            )?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let SocketContent::V1(socket_content_inner) = socket_content;
            let socket = Self::assemble(*socket_id, socket_content_inner.to_owned());

            input_sockets.push((socket, internal_provider));
        }

        // Collect all output sockets with their providers.
        let mut output_sockets: Vec<(Socket, ExternalProvider)> =
            Vec::with_capacity(external_provider_id_map.keys().len());
        for (external_provider_id, external_provider_hash) in external_provider_id_map {
            let external_provider_content = external_provider_content_map
                .get(&external_provider_hash)
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    external_provider_id.into(),
                ))?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let ExternalProviderContent::V1(external_provider_content_inner) =
                external_provider_content;
            let external_provider = ExternalProvider::assemble(
                external_provider_id,
                external_provider_content_inner.to_owned(),
            );

            // Now that we have the provider, assemble the socket.
            let (socket_id, socket_hash) = provider_to_socket_map
                .get(&external_provider_id.into())
                .expect("no socket for provider");
            let socket_content = socket_content_map.get(socket_hash).ok_or(
                WorkspaceSnapshotError::MissingContentFromStore(socket_id.into()),
            )?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let SocketContent::V1(socket_content_inner) = socket_content;
            let socket = Self::assemble(*socket_id, socket_content_inner.to_owned());

            output_sockets.push((socket, external_provider));
        }

        Ok((input_sockets, output_sockets))
    }

    pub fn edge_kind(&self) -> SocketEdgeKind {
        self.edge_kind
    }

    pub fn id(&self) -> SocketId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn diagram_kind(&self) -> DiagramKind {
        self.diagram_kind
    }
}

// impl Socket {
//     pub async fn new(
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//         kind: SocketKind,
//         socket_edge_kind: &SocketEdgeKind,
//         arity: &SocketArity,
//         diagram_kind: &DiagramKind,
//         schema_variant_id: Option<SchemaVariantId>,
//     ) -> SocketResult<Self> {
//         let name = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM socket_create_v1($1, $2, $3, $4, $5, $6, $7)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &name,
//                     &kind.as_ref(),
//                     &socket_edge_kind.as_ref(),
//                     &arity.as_ref(),
//                     &diagram_kind.as_ref(),
//                 ],
//             )
//             .await?;
//         let object: Socket = standard_model::finish_create_from_row(ctx, row).await?;

//         if let Some(schema_variant_id) = schema_variant_id {
//             let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
//                 .await
//                 .map_err(|e| SocketError::SchemaVariant(e.to_string()))?
//                 .ok_or(SocketError::SchemaVariantNotFound(schema_variant_id))?;
//             schema_variant
//                 .add_socket(ctx, &object.id)
//                 .await
//                 .map_err(|e| SocketError::SchemaVariant(e.to_string()))?
//         }

//         Ok(object)
//     }

//     standard_model_accessor!(human_name, Option<String>, SocketResult);
//     standard_model_accessor!(name, String, SocketResult);
//     standard_model_accessor!(kind, Enum(SocketKind), SocketResult);
//     standard_model_accessor!(edge_kind, Enum(SocketEdgeKind), SocketResult);
//     standard_model_accessor!(arity, Enum(SocketArity), SocketResult);
//     standard_model_accessor!(diagram_kind, Enum(DiagramKind), SocketResult);
//     standard_model_accessor!(required, bool, SocketResult);
//     standard_model_accessor!(ui_hidden, bool, SocketResult);

//     standard_model_many_to_many!(
//         lookup_fn: types,
//         associate_fn: add_type,
//         disassociate_fn: remove_type,
//         disassociate_all_fn: remove_all_types,
//         table_name: "socket_many_to_many_schema_variants",
//         left_table: "sockets",
//         left_id: SocketId,
//         right_table: "schema_variants",
//         right_id: SchemaVariantId,
//         which_table_is_this: "left",
//         returns: SchemaVariant,
//         result: SocketResult,
//     );

//     standard_model_belongs_to!(
//         lookup_fn: internal_provider,
//         set_fn: set_internal_provider,
//         unset_fn: unset_internal_provider,
//         table: "socket_belongs_to_internal_provider",
//         model_table: "internal_providers",
//         belongs_to_id: InternalProviderId,
//         returns: InternalProvider,
//         result: SocketResult,
//     );

//     standard_model_belongs_to!(
//         lookup_fn: external_provider,
//         set_fn: set_external_provider,
//         unset_fn: unset_external_provider,
//         table: "socket_belongs_to_external_provider",
//         model_table: "external_providers",
//         belongs_to_id: ExternalProviderId,
//         returns: ExternalProvider,
//         result: SocketResult,
//     );

//     /// Finds the "Frame" [`Socket`] for a given [`Node`](crate::Node) and
//     /// [`SocketEdgeKind`].
//     #[instrument(skip_all)]
//     pub async fn find_frame_socket_for_node(
//         ctx: &DalContext,
//         node_id: NodeId,
//         socket_edge_kind: SocketEdgeKind,
//     ) -> SocketResult<Self> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 FIND_FRAME_SOCKET_FOR_NODE,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &node_id,
//                     &socket_edge_kind.as_ref(),
//                 ],
//             )
//             .await?;
//         Ok(standard_model::object_from_row(row)?)
//     }

//     #[instrument(skip_all)]
//     pub async fn find_for_internal_provider(
//         ctx: &DalContext,
//         internal_provider_id: InternalProviderId,
//     ) -> SocketResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_INTERNAL_PROVIDER,
//                 &[ctx.tenancy(), ctx.visibility(), &internal_provider_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     #[instrument(skip_all)]
//     pub async fn find_for_external_provider(
//         ctx: &DalContext,
//         external_provider_id: ExternalProviderId,
//     ) -> SocketResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_EXTERNAL_PROVIDER,
//                 &[ctx.tenancy(), ctx.visibility(), &external_provider_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// List all [`Sockets`](Self) for the given [`ComponentId`](crate::Component).
//     #[instrument(skip_all)]
//     pub async fn list_for_component(
//         ctx: &DalContext,
//         component_id: ComponentId,
//     ) -> SocketResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_COMPONENT,
//                 &[ctx.tenancy(), ctx.visibility(), &component_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find a [`Socket`] by a provided name for a given [`SocketEdgeKind`] and
//     /// a given [`NodeId`](crate::Node).
//     #[instrument(skip_all)]
//     pub async fn find_by_name_for_edge_kind_and_node(
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//         socket_edge_kind: SocketEdgeKind,
//         node_id: NodeId,
//     ) -> SocketResult<Option<Self>> {
//         let name = name.as_ref();
//         let maybe_row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_BY_NAME_FOR_EDGE_KIND_AND_NODE,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &name,
//                     &socket_edge_kind.as_ref(),
//                     &node_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::option_object_from_row(maybe_row)?)
//     }
// }
