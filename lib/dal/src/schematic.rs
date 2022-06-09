use serde::{Deserialize, Serialize};
use si_data::PgError;
use strum_macros::{AsRefStr, Display, EnumString};
use thiserror::Error;

use crate::edge::{Edge, EdgeId, EdgeKind, VertexObjectKind};
use crate::node::NodeViewKind;
use crate::node_position::NodePositionView;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::socket::SocketId;
use crate::{
    node::NodeId, AttributePrototypeArgument, AttributePrototypeArgumentError, ComponentError,
    ComponentId, DalContext, EdgeError, ExternalProvider, ExternalProviderId, InternalProvider,
    InternalProviderId, Node, NodeError, NodeKind, NodePosition, NodePositionError, NodeTemplate,
    NodeView, PropError, ReadTenancyError, StandardModel, StandardModelError, SystemError,
    SystemId,
};

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component name not found")]
    ComponentNameNotFound,
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
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("system error: {0}")]
    System(#[from] SystemError),
    #[error("system not found")]
    SystemNotFound,
}

pub type SchematicResult<T> = Result<T, SchematicError>;

#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SchematicKind {
    /// Only shows SchemaKind::Concrete and SchemaKind::Implementation
    /// They all have implementation input socket tied to a service output socket (?)
    Component,
    /// Only shows SchemaKind::Concept
    Deployment,
    System,
}

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
        head_explicit_internal_provider_id: Option<InternalProviderId>,
        tail_node_id: &NodeId,
        tail_socket_id: &SocketId,
        tail_external_provider_id: Option<ExternalProviderId>,
    ) -> SchematicResult<Self> {
        let head_node = Node::get_by_id(ctx, head_node_id)
            .await?
            .ok_or(SchematicError::Node(NodeError::NotFound(*head_node_id)))?;
        let tail_node = Node::get_by_id(ctx, tail_node_id)
            .await?
            .ok_or(SchematicError::Node(NodeError::NotFound(*tail_node_id)))?;

        let head_component = head_node
            .component(ctx)
            .await?
            .ok_or(SchematicError::Node(NodeError::ComponentIsNone))?;
        let tail_component = tail_node
            .component(ctx)
            .await?
            .ok_or(SchematicError::Node(NodeError::ComponentIsNone))?;

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
            Err(e) => return Err(SchematicError::Edge(e)),
        };

        // If there is an ExternalProvider corresponding to the tail and an externally-consuming
        // InternalProvider corresponding to the head, then let's "connect" them too.
        if let (Some(tail_external_provider_id), Some(head_explicit_internal_provider_id)) = (
            tail_external_provider_id,
            head_explicit_internal_provider_id,
        ) {
            // TODO(nick): allow for different names.
            let name = "identity".to_string();
            Self::connect_providers(
                ctx,
                name,
                tail_external_provider_id,
                *tail_component.id(),
                head_explicit_internal_provider_id,
                *head_component.id(),
            )
            .await?;
        }

        // TODO: do we have to call Component::resolve_attribute for the head_component here?

        Ok(Self::from_edge(&edge))
    }

    /// This function should be only called by [`Connection::new()`] and integration tests. The
    /// latter is why this function is public.
    pub async fn connect_providers(
        ctx: &DalContext<'_, '_>,
        name: String,
        tail_external_provider_id: ExternalProviderId,
        tail_component_id: ComponentId,
        head_explicit_internal_provider_id: InternalProviderId,
        head_component_id: ComponentId,
    ) -> SchematicResult<()> {
        let tail_external_provider: ExternalProvider =
            ExternalProvider::get_by_id(ctx, &tail_external_provider_id)
                .await?
                .ok_or(SchematicError::ExternalProviderNotFound(
                    tail_external_provider_id,
                ))?;
        let head_explicit_internal_provider: InternalProvider =
            InternalProvider::get_by_id(ctx, &head_explicit_internal_provider_id)
                .await?
                .ok_or(SchematicError::InternalProviderNotFound(
                    head_explicit_internal_provider_id,
                ))?;

        // Check that the explicit internal provider is actually explicit and find its attribute
        // prototype id.
        if *head_explicit_internal_provider.internal_consumer() {
            return Err(SchematicError::FoundImplicitInternalProvider(
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

    pub async fn list(ctx: &DalContext<'_, '_>) -> SchematicResult<Vec<Self>> {
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Schematic {
    nodes: Vec<NodeView>,
    connections: Vec<ConnectionView>,
}

impl Schematic {
    pub async fn find(
        ctx: &DalContext<'_, '_>,
        system_id: Option<SystemId>,
    ) -> SchematicResult<Self> {
        let connections = Connection::list(ctx).await?;
        let mut valid_connections = Vec::new();
        let nodes = Node::list(ctx).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in &nodes {
            // TODO: we have to filter the components here by system

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

            let (schema, kind, name) = match node.kind() {
                NodeKind::Deployment | NodeKind::Component => {
                    let component = node
                        .component(ctx)
                        .await?
                        .ok_or(SchematicError::ComponentNotFound)?;
                    let schema = component
                        .schema(ctx)
                        .await?
                        .ok_or(SchematicError::SchemaNotFound)?;

                    (
                        schema,
                        match node.kind() {
                            NodeKind::Deployment => NodeViewKind::Deployment {
                                component_id: *component.id(),
                            },
                            NodeKind::Component => NodeViewKind::Component {
                                component_id: *component.id(),
                            },
                            NodeKind::System => unreachable!(),
                        },
                        component
                            .find_value_by_json_pointer::<String>(ctx, "/root/si/name")
                            .await?
                            .ok_or(SchematicError::ComponentNameNotFound)?,
                    )
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
                    //     .ok_or(SchematicError::SystemNotFound)?;
                    // let mut tenancy = tenancy.clone();
                    // tenancy.universal = true;
                    // let schema = system
                    //     .schema(ctx)
                    //     .await?
                    //     .ok_or(SchematicError::SchemaNotFound)?;

                    // (schema, system.name().to_owned())
                }
            };

            let positions = NodePosition::find_by_node_id(ctx, system_id, *node.id()).await?;
            let positions = positions.into_iter().map(NodePositionView::from).collect();
            let template = NodeTemplate::new_from_schema_id(ctx, *schema.id()).await?;
            let view = NodeView::new(name, node, kind, positions, template);
            node_views.push(view);
        }

        Ok(Self {
            connections: valid_connections
                .into_iter()
                .filter(|conn| {
                    node_views
                        .iter()
                        .any(|n| conn.destination.node_id == *n.id())
                })
                .map(ConnectionView::from)
                .collect(),
            nodes: node_views,
        })
    }

    pub fn nodes(&self) -> &[NodeView] {
        &self.nodes
    }

    pub fn connections(&self) -> &[ConnectionView] {
        &self.connections
    }
}
