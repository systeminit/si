use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;

use crate::component::ComponentError;
use crate::diagram::node::DiagramComponentView;
use crate::schema::variant::SchemaVariantError;
use crate::{Component, ComponentId, DalContext, HistoryEventError, StandardModelError};

pub mod node;

// pub mod connection;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component status not found for component: {0}")]
    ComponentStatusNotFound(ComponentId),
    #[error("deletion timestamp not found")]
    DeletionTimeStamp,
    #[error("edge not found")]
    EdgeNotFound,
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("node not found")]
    NodeNotFound,
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("position not found")]
    PositionNotFound,
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("socket not found")]
    SocketNotFound,
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
}

pub type DiagramResult<T> = Result<T, DiagramError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagram {
    pub components: Vec<DiagramComponentView>,
    // TODO(nick): restore edges in the diagram.
    // edges: Vec<DiagramEdgeView>,
    pub edges: Vec<()>,
}

impl Diagram {
    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection).
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        // TODO(nick): handle deleted.
        // let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        // TODO(nick): restore listing change status.
        // let modified = ComponentChangeStatus::list_modified(ctx).await?;
        // debug!("modified component change status: {modified:#?}");

        // TODO(nick): restore returning edges.
        // let mut diagram_edges = Vec::new();
        // let edges = Edge::list(ctx).await?;
        // for edge in &edges {
        //     if *edge.kind() == EdgeKind::Configuration {
        //         let change_status = match edge.visibility().change_set_pk {
        //             ChangeSetPk::NONE => ChangeStatus::Unmodified,
        //             _ => ChangeStatus::Added,
        //         };
        //         let conn = Connection::from_edge(edge);
        //         let mut diagram_edge_view =
        //             DiagramEdgeView::from_with_change_status(conn, change_status);
        //         diagram_edge_view.set_actor_details(ctx, edge).await?;
        //         diagram_edges.push(diagram_edge_view);
        //     }
        // }
        //
        // let deleted_edges: Vec<Edge> = EdgeChangeStatus::list_deleted(ctx).await?;
        //
        // let deleted_diagram_edges = ctx
        //     .run_with_deleted_visibility(|ctx_with_deleted| async move {
        //         let mut deleted_diagram_edges = Vec::new();
        //
        //         for deleted_edge in deleted_edges {
        //             if *deleted_edge.kind() == EdgeKind::Configuration {
        //                 let conn = Connection::from_edge(&deleted_edge);
        //                 let mut diagram_edge_view =
        //                     DiagramEdgeView::from_with_change_status(conn, ChangeStatus::Deleted);
        //                 diagram_edge_view
        //                     .set_actor_details(&ctx_with_deleted, &deleted_edge)
        //                     .await?;
        //                 deleted_diagram_edges.push(diagram_edge_view);
        //             }
        //         }
        //
        //         Ok::<_, DiagramError>(deleted_diagram_edges)
        //     })
        //     .await?;
        //
        // diagram_edges.extend(deleted_diagram_edges);

        // TODO(nick): ensure we can show both deleted and exiting nodes.
        // let nodes = ctx
        //     .run_with_deleted_visibility(|ctx_with_deleted| async move {
        //         Node::list_live(&ctx_with_deleted, NodeKind::Configuration).await
        //     })
        //     .await?;
        let components = Component::list(ctx).await?;

        let mut component_views = Vec::with_capacity(components.len());
        for component in &components {
            let schema_variant = Component::schema_variant(ctx, component.id()).await?;

            // TODO(nick): restore this.
            let is_modified = false;
            // let is_modified = modified
            //     .clone()
            //     .iter()
            //     .any(|s| s.component_id == *component.id());

            // TODO(nick): restore frames.
            // // Get Parent Id
            // let sockets = SocketView::list(ctx, &schema_variant).await?;
            // let maybe_socket_to_parent = sockets.iter().find(|socket| {
            //     socket.label == "Frame" && socket.direction == SocketDirection::Output
            // });
            //
            // let edges_with_deleted =
            //     Edge::list_for_component(ctx_with_deleted, *component.id()).await?;
            //
            // let mut parent_node_id = None;
            //
            // if let Some(socket_to_parent) = maybe_socket_to_parent {
            //     for edge in &edges_with_deleted {
            //         if edge.tail_node_id() == *node.id()
            //             && edge.tail_socket_id().to_string() == socket_to_parent.id
            //             && (edge.visibility().deleted_at.is_none() || edge.deleted_implicitly())
            //         {
            //             parent_node_id = Some(edge.head_node_id());
            //             break;
            //         }
            //     }
            // };
            //
            // // Get Child Ids
            // let maybe_socket_from_children = sockets.iter().find(|socket| {
            //     socket.label == "Frame" && socket.direction == SocketDirection::Input
            // });
            //
            // let mut child_node_ids = vec![];
            // if let Some(socket_from_children) = maybe_socket_from_children {
            //     for edge in &edges_with_deleted {
            //         if edge.head_node_id() == *node.id()
            //             && edge.head_socket_id().to_string() == socket_from_children.id
            //             && (edge.visibility().deleted_at.is_none()
            //                 || (edge.deleted_implicitly() && edge.visibility().in_change_set()))
            //         {
            //             let child_node = Node::get_by_id(ctx_with_deleted, &edge.tail_node_id())
            //                 .await?
            //                 .ok_or(DiagramError::NodeNotFound)?;
            //
            //             // This is a node in the current changeset and it is not deleted
            //             if child_node.visibility().in_change_set()
            //                 && child_node.visibility().deleted_at.is_none()
            //             {
            //                 child_node_ids.push(edge.tail_node_id());
            //                 continue;
            //             }
            //
            //             // this is a node in the current changeset that has been marked as deleted
            //             // now we need to check to see if it is exists in head
            //             // if it does, then it's a ghosted node and should be included as a child
            //             if child_node.visibility().in_change_set()
            //                 && child_node.visibility().deleted_at.is_some()
            //             {
            //                 let head_ctx = &ctx.clone_with_head();
            //                 let head_node = Node::get_by_id(head_ctx, &edge.tail_node_id()).await?;
            //                 if head_node.is_some() {
            //                     child_node_ids.push(edge.tail_node_id());
            //                     continue;
            //                 }
            //             }
            //
            //             // if the node is in head, doesn't exist directly on the changeset
            //             // and not marked as deleted in head, then it's also a valid child
            //             // *Remember*: a node won't exist in the changeset until a change is
            //             // made to a node!!
            //             if child_node.visibility().is_head()
            //                 && child_node.visibility().deleted_at.is_none()
            //             {
            //                 child_node_ids.push(edge.tail_node_id());
            //                 continue;
            //             }
            //         }
            //     }
            // };

            let parent_component_id = None;
            let child_component_ids = Vec::new();

            let view = DiagramComponentView::new(
                ctx,
                // TODO(nick): handle deleted.
                // ctx_with_deleted,
                &component,
                parent_component_id,
                child_component_ids,
                is_modified,
                &schema_variant,
            )
            .await?;
            component_views.push(view);
        }

        // TODO(nick): restore the ability to show edges.
        Ok(Self {
            edges: Vec::new(),
            components: component_views,
        })
    }
}
