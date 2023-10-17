use content_store::ContentHash;
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};
use telemetry::prelude::*;


use si_pkg::SocketSpecArity;

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{
    label_list::ToLabelList, pk, StandardModel, Timestamp,
};

// const FIND_BY_NAME_FOR_EDGE_KIND_AND_NODE: &str =
//     include_str!("queries/socket/find_by_name_for_edge_kind_and_node.sql");
// const FIND_FRAME_SOCKET_FOR_NODE: &str =
//     include_str!("queries/socket/find_frame_socket_for_node.sql");
// const LIST_FOR_COMPONENT: &str = include_str!("queries/socket/list_for_component.sql");
// const FIND_FOR_INTERNAL_PROVIDER: &str =
//     include_str!("queries/socket/find_for_internal_provider.sql");
// const FIND_FOR_EXTERNAL_PROVIDER: &str =
//     include_str!("queries/socket/find_for_external_provider.sql");

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

#[derive(Debug, PartialEq)]
pub struct SocketGraphNode {
    id: SocketId,
    content_address: ContentAddress,
    content: SocketContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum SocketContent {
    V1(SocketContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SocketContentV1 {
    #[serde(flatten)]
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

impl SocketGraphNode {
    pub fn assemble(
        id: impl Into<SocketId>,
        content_hash: ContentHash,
        content: SocketContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::Socket(content_hash),
            content,
        }
    }
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
