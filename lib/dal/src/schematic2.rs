use serde::{Deserialize, Serialize};
use si_data::PgError;
use std::num::ParseIntError;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use thiserror::Error;

use crate::edge::{Edge, EdgeId, EdgeKind, VertexObjectKind};
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::socket::{SocketArity, SocketEdgeKind, SocketId};
use crate::{
    node::NodeId, schema::variant::SchemaVariantError, AttributePrototypeArgument,
    AttributePrototypeArgumentError, ComponentError, ComponentId, DalContext, EdgeError,
    ExternalProvider, ExternalProviderId, InternalProvider, InternalProviderId, Node, NodeError,
    NodeKind, NodePosition, NodePositionError, PropError, ReadTenancyError, SchemaVariant,
    StandardModel, StandardModelError, SystemError, SystemId,
};

#[derive(Error, Debug)]
pub enum SchematicError2 {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("external provider not found for id: {0}")]
    ExternalProviderNotFound(ExternalProviderId),
    #[error("implicit internal provider cannot be used for inter component connection: {0}")]
    FoundImplicitInternalProvider(InternalProviderId),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider not found for id: {0}")]
    InternalProviderNotFound(InternalProviderId),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("position not found")]
    PositionNotFound,
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("system error: {0}")]
    System(#[from] SystemError),
    #[error("system not found")]
    SystemNotFound,
}

pub type SchematicResult2<T> = Result<T, SchematicError2>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    id: EdgeId,
    pub classification: EdgeKind,
    pub source: Vertex,
    pub destination: Vertex,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Vertex {
    pub node_id: NodeId,
    pub socket_id: SocketId,
}

impl Connection {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        head_node_id: &NodeId,
        head_socket_id: &SocketId,
        head_explicit_internal_provider_id: InternalProviderId,
        tail_node_id: &NodeId,
        tail_socket_id: &SocketId,
        tail_external_provider_id: ExternalProviderId,
    ) -> SchematicResult2<Self> {
        let head_node = Node::get_by_id(ctx, head_node_id)
            .await?
            .ok_or(SchematicError2::Node(NodeError::NotFound(*head_node_id)))?;
        let tail_node = Node::get_by_id(ctx, tail_node_id)
            .await?
            .ok_or(SchematicError2::Node(NodeError::NotFound(*tail_node_id)))?;

        let head_component = head_node
            .component(ctx)
            .await?
            .ok_or(SchematicError2::Node(NodeError::ComponentIsNone))?;
        let tail_component = tail_node
            .component(ctx)
            .await?
            .ok_or(SchematicError2::Node(NodeError::ComponentIsNone))?;

        // TODO(nick): allow for non-identity inter component connections.
        Self::connect_providers(
            ctx,
            "identity",
            tail_external_provider_id,
            *tail_component.id(),
            head_explicit_internal_provider_id,
            *head_component.id(),
        )
        .await?;

        // TODO(nick): a lot of hardcoded values here along with the (temporary) insinuation that an
        // edge is equivalent to a connection.
        let edge = match Edge::new(
            ctx,
            EdgeKind::Configures,
            *head_node_id,
            VertexObjectKind::Component,
            (*head_component.id()).into(),
            *head_socket_id,
            *tail_node_id,
            VertexObjectKind::Component,
            (*tail_component.id()).into(),
            *tail_socket_id,
        )
        .await
        {
            Ok(edge) => edge,
            Err(e) => return Err(SchematicError2::Edge(e)),
        };

        // TODO: do we have to call Component::resolve_attribute for the head_component here?

        Ok(Self::from_edge(&edge))
    }

    /// This function should be only called by [`Connection::new()`] and integration tests. The
    /// latter is why this function is public.
    pub async fn connect_providers(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        tail_external_provider_id: ExternalProviderId,
        tail_component_id: ComponentId,
        head_explicit_internal_provider_id: InternalProviderId,
        head_component_id: ComponentId,
    ) -> SchematicResult2<()> {
        let tail_external_provider: ExternalProvider =
            ExternalProvider::get_by_id(ctx, &tail_external_provider_id)
                .await?
                .ok_or(SchematicError2::ExternalProviderNotFound(
                    tail_external_provider_id,
                ))?;
        let head_explicit_internal_provider: InternalProvider =
            InternalProvider::get_by_id(ctx, &head_explicit_internal_provider_id)
                .await?
                .ok_or(SchematicError2::InternalProviderNotFound(
                    head_explicit_internal_provider_id,
                ))?;

        // Check that the explicit internal provider is actually explicit and find its attribute
        // prototype id.
        if head_explicit_internal_provider.is_internal_consumer() {
            return Err(SchematicError2::FoundImplicitInternalProvider(
                *head_explicit_internal_provider.id(),
            ));
        }
        let head_explicit_internal_provider_attribute_prototype = head_explicit_internal_provider
            .attribute_prototype_id()
            .ok_or(InternalProviderError::EmptyAttributePrototype)?;

        // Now, we can create the inter component attribute prototype argument.
        AttributePrototypeArgument::new_for_inter_component(
            ctx,
            *head_explicit_internal_provider_attribute_prototype,
            name,
            *tail_external_provider.id(),
            tail_component_id,
            head_component_id,
        )
        .await?;
        Ok(())
    }

    pub async fn list(ctx: &DalContext<'_, '_>) -> SchematicResult2<Vec<Self>> {
        let edges = Edge::list(ctx).await?;
        let connections = edges.iter().map(Self::from_edge).collect::<Vec<Self>>();
        Ok(connections)
    }

    pub fn from_edge(edge: &Edge) -> Self {
        Self {
            id: *edge.id(),
            classification: edge.kind().clone(),
            source: Vertex {
                node_id: edge.head_node_id(),
                socket_id: edge.head_socket_id(),
            },
            destination: Vertex {
                node_id: edge.tail_node_id(),
                socket_id: edge.tail_socket_id(),
            },
        }
    }

    pub fn source(&self) -> (NodeId, SocketId) {
        (self.source.node_id, self.source.socket_id)
    }

    pub fn destination(&self) -> (NodeId, SocketId) {
        (self.destination.node_id, self.destination.socket_id)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionView {
    source_node_id: NodeId,
    source_socket_id: SocketId,
    destination_node_id: NodeId,
    destination_socket_id: SocketId,
}

impl From<Connection> for ConnectionView {
    fn from(conn: Connection) -> Self {
        Self {
            source_node_id: conn.source.node_id,
            source_socket_id: conn.source.socket_id,
            destination_node_id: conn.destination.node_id,
            destination_socket_id: conn.destination.socket_id,
        }
    }
}

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
pub enum SocketDirection2 {
    Input,
    Output,
    Bidirectional,
}

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
pub enum NodeSide2 {
    Left,
    Right,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocketView2 {
    id: String,
    label: String,
    #[serde(rename = "type")]
    ty: String,
    direction: SocketDirection2,
    max_connections: Option<usize>,
    is_required: Option<bool>,
    node_side: NodeSide2,
}

impl SocketView2 {
    pub async fn list(
        ctx: &DalContext<'_, '_>,
        node: &Node,
        schema_variant: &SchemaVariant,
    ) -> SchematicResult2<Vec<Self>> {
        Ok(schema_variant
            .sockets(ctx)
            .await?
            .into_iter()
            .filter(|socket| socket.name() != "includes")
            .map(|socket| Self {
                id: format!("{}-{}", node.id(), socket.id()),
                label: socket.name().to_owned(),
                ty: socket.name().to_owned(),
                // Note: it's not clear if this mapping is correct, and there is no backend support for bidirectional sockets for now
                direction: match socket.edge_kind() {
                    SocketEdgeKind::Output => SocketDirection2::Output,
                    _ => SocketDirection2::Input,
                },
                max_connections: match socket.arity() {
                    SocketArity::Many => None,
                    SocketArity::One => Some(1),
                },
                is_required: Some(socket.required()),
                node_side: match socket.edge_kind() {
                    SocketEdgeKind::Output => NodeSide2::Right,
                    _ => NodeSide2::Left,
                },
            })
            .collect())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint2 {
    x: isize,
    y: isize,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeView2 {
    id: String,
    #[serde(rename = "type")]
    ty: Option<String>,
    title: String,
    subtitle: Option<String>,
    content: Option<String>,
    sockets: Option<Vec<SocketView2>>,
    position: GridPoint2,
    color: Option<String>,
}

impl NodeView2 {
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        node: &Node,
        position: &NodePosition,
        schema_variant: &SchemaVariant,
    ) -> SchematicResult2<Self> {
        let component = node
            .component(ctx)
            .await?
            .ok_or(SchematicError2::ComponentNotFound)?;
        Ok(Self {
            id: node.id().to_string(),
            ty: None,
            title: schema_variant
                .schema(ctx)
                .await?
                .ok_or(SchematicError2::SchemaNotFound)?
                .name()
                .to_owned(),
            subtitle: component
                .find_value_by_json_pointer(ctx, "/root/si/name")
                .await?,
            content: None,
            sockets: Some(SocketView2::list(ctx, node, schema_variant).await?),
            position: GridPoint2 {
                x: position.x().parse()?,
                y: position.y().parse()?,
            },
            color: schema_variant
                .color()
                .map(|color_int| format!("#{color_int:x}")),
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EdgeView2 {
    id: String,
    #[serde(rename = "type")]
    ty: Option<String>,
    name: Option<String>,
    from_socket_id: String,
    to_socket_id: String,
    is_bidirectional: Option<bool>,
}

impl From<Connection> for EdgeView2 {
    fn from(conn: Connection) -> Self {
        Self {
            id: conn.id.to_string(),
            ty: None,
            name: None,
            from_socket_id: format!("{}-{}", conn.source.node_id, conn.source.socket_id),
            to_socket_id: format!(
                "{}-{}",
                conn.destination.node_id, conn.destination.socket_id
            ),
            is_bidirectional: Some(false),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Schematic2 {
    nodes: Vec<NodeView2>,
    edges: Vec<EdgeView2>,
}

impl Schematic2 {
    pub async fn find(
        ctx: &DalContext<'_, '_>,
        system_id: Option<SystemId>,
    ) -> SchematicResult2<Self> {
        let connections = Connection::list(ctx).await?;
        let mut valid_connections = Vec::new();
        let nodes = Node::list(ctx).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in &nodes {
            // TODO: we have to filter the components here by system

            // hide deployment nodes from the diagram since we'll likely remove it completely
            if *node.kind() == NodeKind::Deployment {
                continue;
            }

            // Allows us to ignore nodes that aren't in current application
            let conns = connections
                .iter()
                .filter(|c| c.source.node_id == *node.id());

            let is_from_this_schematic = Some(*node.id()) == ctx.application_node_id()
                || conns
                    .clone()
                    .any(|conn| Some(conn.destination.node_id) == ctx.application_node_id());
            if !is_from_this_schematic {
                continue;
            }
            valid_connections.extend(conns.cloned());

            let schema_variant = match node.kind() {
                NodeKind::Deployment | NodeKind::Component => {
                    let component = node
                        .component(ctx)
                        .await?
                        .ok_or(SchematicError2::ComponentNotFound)?;
                    component
                        .schema_variant(ctx)
                        .await?
                        .ok_or(SchematicError2::SchemaVariantNotFound)?
                }
                NodeKind::System => {
                    // We're going to skip all `NodeKind::System` nodes
                    continue;

                    // TODO(fnichol): We were failing in `node.system()` with an `Error: dal
                    // schematic error: system not found` error. For the moment we're going to
                    // filter out system-backed nodes, but ultimately we might want to return all
                    // node kinds back to the frontend for use.
                    //
                    // let system = node
                    //     .system(ctx)
                    //     .await?
                    //     .ok_or(SchematicError2::SystemNotFound)?;
                    // let mut tenancy = tenancy.clone();
                    // tenancy.universal = true;
                    // let schema = system
                    //     .schema(ctx)
                    //     .await?
                    //     .ok_or(SchematicError2::SchemaNotFound)?;

                    // (schema, system.name().to_owned())
                }
            };

            let positions = NodePosition::find_by_node_id(ctx, system_id, *node.id()).await?;
            let position = match positions.into_iter().next() {
                Some(pos) => pos,
                None => continue, // Note: do we want to ignore things with no position?
            };
            let view = NodeView2::new(ctx, node, &position, &schema_variant).await?;
            node_views.push(view);
        }

        Ok(Self {
            edges: valid_connections
                .into_iter()
                .filter(|conn| {
                    node_views
                        .iter()
                        .any(|n| conn.destination.node_id.to_string() == n.id)
                })
                .map(EdgeView2::from)
                .collect(),
            nodes: node_views,
        })
    }

    pub fn nodes(&self) -> &[NodeView2] {
        &self.nodes
    }

    pub fn edges(&self) -> &[EdgeView2] {
        &self.edges
    }
}
