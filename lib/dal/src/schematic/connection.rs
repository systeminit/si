use serde::{Deserialize, Serialize};

use crate::edge::{Edge, EdgeId, EdgeKind, VertexObjectKind};
use crate::provider::internal::InternalProviderError;
use crate::schematic::SchematicResult;
use crate::socket::SocketId;
use crate::{
    node::NodeId, AttributePrototypeArgument, ComponentId, DalContext, ExternalProvider,
    ExternalProviderId, InternalProvider, InternalProviderId, Node, NodeError, SchematicError,
    StandardModel,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Vertex {
    pub node_id: NodeId,
    pub socket_id: SocketId,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: EdgeId,
    pub classification: EdgeKind,
    pub source: Vertex,
    pub destination: Vertex,
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
            Err(e) => return Err(SchematicError::Edge(e)),
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
        if head_explicit_internal_provider.is_internal_consumer() {
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
                node_id: edge.tail_node_id(),
                socket_id: edge.tail_socket_id(),
            },
            destination: Vertex {
                node_id: edge.head_node_id(),
                socket_id: edge.head_socket_id(),
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
pub struct SchematicEdgeView {
    id: String,
    #[serde(rename = "type")]
    ty: Option<String>,
    name: Option<String>,
    from_socket_id: String,
    to_socket_id: String,
    is_bidirectional: Option<bool>,
}

impl SchematicEdgeView {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<Connection> for SchematicEdgeView {
    fn from(conn: Connection) -> Self {
        let source_node_id: i64 = conn.source.node_id.into();
        let source_socket_id: i64 = conn.source.socket_id.into();
        let destination_node_id: i64 = conn.destination.node_id.into();
        let destination_socket_id: i64 = conn.destination.socket_id.into();
        let connection_id: i64 = conn.id.into();
        Self {
            id: connection_id.to_string(),
            ty: None,
            name: None,
            from_socket_id: format!("{}-{}", source_node_id, source_socket_id),
            to_socket_id: format!("{}-{}", destination_node_id, destination_socket_id),
            is_bidirectional: Some(false),
        }
    }
}
