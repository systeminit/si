use serde::{Deserialize, Serialize};

use crate::edge::{Edge, EdgeId, EdgeKind};

use crate::change_status::ChangeStatus;
use crate::diagram::DiagramResult;
use crate::job::definition::DependentValuesUpdate;
use crate::socket::SocketId;
use crate::{
    node::NodeId, AttributePrototypeArgument, AttributeReadContext, AttributeValue, DalContext,
    DiagramError, ExternalProviderId, Node, PropId, Socket, StandardModel,
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
        ctx: &DalContext,
        from_node_id: NodeId,
        from_socket_id: SocketId,
        to_node_id: NodeId,
        to_socket_id: SocketId,
        edge_kind: EdgeKind,
    ) -> DiagramResult<Self> {
        let edge = Edge::new_for_connection(
            ctx,
            to_node_id,
            to_socket_id,
            from_node_id,
            from_socket_id,
            edge_kind,
        )
        .await?;
        Ok(Connection::from_edge(&edge))
    }

    pub async fn list(ctx: &DalContext) -> DiagramResult<Vec<Self>> {
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

    pub async fn delete_for_edge(ctx: &DalContext, edge_id: EdgeId) -> DiagramResult<()> {
        let edge = Edge::get_by_id(ctx, &edge_id)
            .await?
            .ok_or(DiagramError::EdgeNotFound)?;

        let head_component_id = *{
            let head_node = Node::get_by_id(ctx, &edge.head_node_id())
                .await?
                .ok_or(DiagramError::NodeNotFound)?;
            head_node
                .component(ctx)
                .await?
                .ok_or(DiagramError::ComponentNotFound)?
                .id()
        };

        let tail_component_id = *{
            let tail_node = Node::get_by_id(ctx, &edge.tail_node_id())
                .await?
                .ok_or(DiagramError::NodeNotFound)?;
            tail_node
                .component(ctx)
                .await?
                .ok_or(DiagramError::ComponentNotFound)?
                .id()
        };

        // This code assumes that every connection is established between a tail external provider and
        // a head (explicit) internal provider. That might not be the case, but it true in practice for the present state of the interface
        // (aggr frame connection to children shouldn't go through this path)
        let external_provider = {
            let socket = Socket::get_by_id(ctx, &edge.tail_socket_id())
                .await?
                .ok_or(DiagramError::SocketNotFound)?;

            socket
                .external_provider(ctx)
                .await?
                .ok_or_else(|| DiagramError::ExternalProviderNotFoundForSocket(*socket.id()))?
        };

        let internal_provider_id = *{
            let socket = Socket::get_by_id(ctx, &edge.head_socket_id())
                .await?
                .ok_or(DiagramError::SocketNotFound)?;

            socket
                .internal_provider(ctx)
                .await?
                .ok_or_else(|| DiagramError::InternalProviderNotFoundForSocket(*socket.id()))?
                .id()
        };

        // Delete the arguments that have the same external provider of the edge, and are connected to an attribute prototype for
        let edge_argument = AttributePrototypeArgument::find_for_providers_and_components(
            ctx,
            external_provider.id(),
            &internal_provider_id,
            &tail_component_id,
            &head_component_id,
        )
        .await?
        .ok_or(DiagramError::AttributePrototypeNotFound)?;

        edge_argument.delete_by_id(ctx).await?;
        edge.delete_by_id(ctx).await?;

        let read_context = AttributeReadContext {
            prop_id: Some(PropId::NONE),
            internal_provider_id: Some(internal_provider_id),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(head_component_id),
        };

        let mut attr_value = AttributeValue::find_for_context(ctx, read_context)
            .await?
            .ok_or(DiagramError::AttributeValueNotFound)?;

        let sibling_arguments = AttributePrototypeArgument::find_by_attr(
            ctx,
            "external_provider_id",
            external_provider.id(),
        )
        .await?;

        let arg_count = sibling_arguments.len();

        if arg_count > 0 {
            attr_value.update_from_prototype_function(ctx).await?;

            ctx.enqueue_job(DependentValuesUpdate::new(ctx, vec![*attr_value.id()]))
                .await;
        } else {
            let attr_val_context = attr_value.context;
            attr_value.unset_attribute_prototype(ctx).await?;

            attr_value.delete_by_id(ctx).await?;

            let att_val = AttributeValue::find_for_context(ctx, attr_val_context.into())
                .await?
                .ok_or(DiagramError::AttributeValueNotFound)?;

            ctx.enqueue_job(DependentValuesUpdate::new(ctx, vec![*att_val.id()]))
                .await;
        }

        Ok(())
    }

    pub async fn restore_for_edge(ctx: &DalContext, edge_id: EdgeId) -> DiagramResult<()> {
        Edge::restore_by_id(ctx, edge_id).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramEdgeView {
    id: String,
    #[serde(rename = "type")]
    ty: Option<String>,
    name: Option<String>,
    from_node_id: String,
    from_socket_id: String,
    to_node_id: String,
    to_socket_id: String,
    is_bidirectional: Option<bool>,
    change_status: ChangeStatus,
}

impl DiagramEdgeView {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl DiagramEdgeView {
    pub fn from_with_change_status(conn: Connection, change_status: ChangeStatus) -> Self {
        Self {
            id: conn.id.to_string(),
            ty: None,
            name: None,
            from_node_id: conn.source.node_id.to_string(),
            from_socket_id: conn.source.socket_id.to_string(),
            to_node_id: conn.destination.node_id.to_string(),
            to_socket_id: conn.destination.socket_id.to_string(),
            is_bidirectional: Some(false),
            change_status,
        }
    }
}

impl From<Connection> for DiagramEdgeView {
    fn from(conn: Connection) -> Self {
        DiagramEdgeView::from_with_change_status(conn, ChangeStatus::Unmodified)
    }
}
