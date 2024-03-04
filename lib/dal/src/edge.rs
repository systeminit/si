//! This module contains [`Edge`], the mathematical "edge" between two [`Nodes`](crate::Node) in a
//! graph.

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use strum::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::argument::FuncArgumentError;
use crate::node::NodeId;
use crate::standard_model::objects_from_rows;
use crate::{
    impl_standard_model, pk, socket::SocketId, standard_model, standard_model_accessor,
    AttributeValue, ComponentId, Func, HistoryActor, HistoryEventError, Node, PropId, Socket,
    StandardModel, StandardModelError, Tenancy, Timestamp, UserPk, Visibility,
};
use crate::{
    Component, DalContext, ExternalProvider, ExternalProviderId, InternalProvider,
    InternalProviderId, TransactionsError,
};

const LIST_PARENTS_FOR_COMPONENT: &str =
    include_str!("queries/edge/list_parents_for_component.sql");
const LIST_CHILDREN_FOR_NODE: &str = include_str!("queries/edge/list_children_for_node.sql");
const LIST_CHILDREN_FOR_COMPONENT: &str =
    include_str!("queries/edge/list_children_for_component.sql");
const LIST_FOR_COMPONENT: &str = include_str!("queries/edge/list_for_component.sql");
const LIST_FOR_KIND: &str = include_str!("queries/edge/list_for_kind.sql");
const FIND_DELETED_EQUIVALENT: &str = include_str!("queries/edge/find_deleted_equivalent.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("component error: {0}")]
    Component(String),
    #[error("cannot find component for id: {0}")]
    ComponentNotFound(ComponentId),
    #[error("cannot find component for node id: {0}")]
    ComponentNotFoundForNode(NodeId),
    #[error("edge not found for id: {0}")]
    EdgeNotFound(EdgeId),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("external provider not found for id: {0}")]
    ExternalProviderNotFound(ExternalProviderId),
    #[error("external provider not found for socket id: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("implicit internal provider cannot be used for inter component connection: {0}")]
    FoundImplicitInternalProvider(InternalProviderId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider not found for id: {0}")]
    InternalProviderNotFound(InternalProviderId),
    #[error("internal provider not found for socket id: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("cannot find node id: {0}")]
    NodeNotFound(NodeId),
    #[error("cannot find parent component for id: {0}")]
    ParentComponentNotFound(ComponentId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("cannot restore edge ({0}) to deleted node: {1}")]
    RestoringAnEdgeToDeletedNode(EdgeId, NodeId),
    #[error("cannot restore non deleted edge with id: {0}")]
    RestoringNonDeletedEdge(EdgeId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("cannot find socket id: {0}")]
    SocketNotFound(SocketId),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("summary diagram error: {0}")]
    SummaryDiagram(String),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type EdgeResult<T> = Result<T, EdgeError>;

/// Used to dictate what [`EdgeKinds`](EdgeKind) can be for the head and tail of an [`Edges`](Edge).
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum VertexObjectKind {
    /// Used for [`Nodes`](crate::Node) of [`NodeKind::Configuration`](crate::NodeKind::Configuration).
    Configuration,
}

/// The kind of an [`Edge`](Edge). This provides the ability to categorize [`Edges`](Edge)
/// and create [`EdgeKind`](Self)-specific graphs.
#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display, EnumString, AsRefStr, Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EdgeKind {
    /// Used to connect a configuration to another configuration.
    Configuration,
    Symbolic,
}

pk!(EdgeId);
pk!(EdgePk);

/// A mathematical edge between a head and a tail [`Node`](crate::Node).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pk: EdgePk,
    id: EdgeId,
    kind: EdgeKind,
    // NOTE: Would love to flatten this, but serde doesn't allow flatten and rename.
    head_node_id: NodeId,
    head_object_kind: VertexObjectKind,
    head_object_id: EdgeObjectId,
    head_socket_id: SocketId,
    tail_node_id: NodeId,
    tail_object_kind: VertexObjectKind,
    tail_object_id: EdgeObjectId,
    tail_socket_id: SocketId,
    creation_user_pk: Option<UserPk>,
    deletion_user_pk: Option<UserPk>,
    deleted_implicitly: bool,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Edge,
    pk: EdgePk,
    id: EdgeId,
    table_name: "edges",
    history_event_label_base: "edge",
    history_event_message_name: "Edge"
}

pk!(EdgeObjectId);

impl From<EdgeObjectId> for ComponentId {
    fn from(id: EdgeObjectId) -> Self {
        Self::from(id.into_inner())
    }
}

impl From<ComponentId> for EdgeObjectId {
    fn from(id: ComponentId) -> Self {
        Self::from(id.into_inner())
    }
}

impl Edge {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        kind: EdgeKind,
        head_node_id: NodeId,
        head_object_kind: VertexObjectKind,
        head_object_id: EdgeObjectId,
        head_socket_id: SocketId,
        tail_node_id: NodeId,
        tail_object_kind: VertexObjectKind,
        tail_object_id: EdgeObjectId,
        tail_socket_id: SocketId,
    ) -> EdgeResult<Self> {
        let actor_user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => Some(*user_pk),
            _ => None,
        };

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM edge_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &kind.to_string(),
                    &head_node_id,
                    &head_object_kind.to_string(),
                    &head_object_id,
                    &head_socket_id,
                    &tail_node_id,
                    &tail_object_kind.to_string(),
                    &tail_object_id,
                    &tail_socket_id,
                    &actor_user_pk,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        diagram::summary_diagram::create_edge_entry(ctx, &object)
            .await
            .map_err(|e| EdgeError::SummaryDiagram(e.to_string()))?;
        Ok(object)
    }

    /// Creates a new [`Edge`](Self) and "connects" the underlying [`providers`](crate::provider)
    /// for the [`Connection`](crate::Connection).
    ///
    /// Terminology:
    /// - The _from_ side of a [`Connection`](crate::Connection) is the _tail_ of an [`Edge`](Self)
    /// - The _to_ side of a [`Connection`](crate::Connection) is the _head_ of an [`Edge`](Self)
    ///
    /// Please note that the _head_ information comes before the _tail_ information in the
    /// function parameters.
    #[allow(clippy::too_many_arguments)]
    pub async fn new_for_connection(
        ctx: &DalContext,
        head_node_id: NodeId,
        head_socket_id: SocketId,
        tail_node_id: NodeId,
        tail_socket_id: SocketId,
        edge_kind: EdgeKind,
    ) -> EdgeResult<Self> {
        // Revive edge if it already exists
        if let Some(equivalent_edge) = {
            let row = ctx
                .txns()
                .await?
                .pg()
                .query_opt(
                    FIND_DELETED_EQUIVALENT,
                    &[
                        ctx.tenancy(),
                        &ctx.visibility().change_set_pk,
                        &head_node_id,
                        &head_socket_id,
                        &tail_node_id,
                        &tail_socket_id,
                    ],
                )
                .await?;
            standard_model::object_option_from_row_option::<Edge>(row)?
        } {
            if let Some(restored_edge) = Self::restore_by_id(ctx, equivalent_edge.id).await? {
                return Ok(restored_edge);
            }
        }

        // Otherwise create a new one
        let head_component = Component::find_for_node(ctx, head_node_id)
            .await
            .map_err(|err| EdgeError::Component(err.to_string()))?
            .ok_or(NodeError::ComponentIsNone)?;
        let tail_component = Component::find_for_node(ctx, tail_node_id)
            .await
            .map_err(|err| EdgeError::Component(err.to_string()))?
            .ok_or(NodeError::ComponentIsNone)?;

        let head_explicit_internal_provider =
            InternalProvider::find_explicit_for_socket(ctx, head_socket_id)
                .await?
                .ok_or(EdgeError::InternalProviderNotFoundForSocket(head_socket_id))?;
        let tail_external_provider = ExternalProvider::find_for_socket(ctx, tail_socket_id)
            .await?
            .ok_or(EdgeError::ExternalProviderNotFoundForSocket(tail_socket_id))?;

        // We don't want to connect the provider when we are not using configuration edge kind
        if edge_kind == EdgeKind::Configuration {
            // TODO(nick): allow for more transformation functions.
            Self::connect_providers_for_components(
                ctx,
                *head_explicit_internal_provider.id(),
                *head_component.id(),
                *tail_external_provider.id(),
                *tail_component.id(),
            )
            .await?;
        }

        // NOTE(nick): a lot of hardcoded values here that'll likely need to be adjusted.
        let edge = Edge::new(
            ctx,
            edge_kind,
            head_node_id,
            VertexObjectKind::Configuration,
            EdgeObjectId::from(*head_component.id()),
            head_socket_id,
            tail_node_id,
            VertexObjectKind::Configuration,
            EdgeObjectId::from(*tail_component.id()),
            tail_socket_id,
        )
        .await?;
        Ok(edge)
    }

    standard_model_accessor!(kind, Enum(EdgeKind), EdgeResult);

    // Sockets
    standard_model_accessor!(head_node_id, Pk(NodeId), EdgeResult);
    standard_model_accessor!(head_object_kind, Enum(VertexObjectKind), EdgeResult);
    standard_model_accessor!(head_object_id, Pk(EdgeObjectId), EdgeResult);
    standard_model_accessor!(head_socket_id, Pk(SocketId), EdgeResult);
    standard_model_accessor!(tail_node_id, Pk(NodeId), EdgeResult);
    standard_model_accessor!(tail_object_kind, Enum(VertexObjectKind), EdgeResult);
    standard_model_accessor!(tail_object_id, Pk(EdgeObjectId), EdgeResult);
    standard_model_accessor!(tail_socket_id, Pk(SocketId), EdgeResult);
    standard_model_accessor!(creation_user_pk, Option<Pk(UserPk)>, EdgeResult);
    standard_model_accessor!(deletion_user_pk, Option<Pk(UserPk)>, EdgeResult);
    standard_model_accessor!(deleted_implicitly, bool, EdgeResult);

    pub async fn list_children_for_node(
        ctx: &DalContext,
        node_id: NodeId,
    ) -> EdgeResult<Vec<NodeId>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_CHILDREN_FOR_NODE,
                &[ctx.tenancy(), ctx.visibility(), &node_id],
            )
            .await?;
        let objects = rows.into_iter().map(|row| row.get("node_id")).collect();
        Ok(objects)
    }

    pub async fn list_children_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> EdgeResult<Vec<ComponentId>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_CHILDREN_FOR_COMPONENT,
                &[ctx.tenancy(), ctx.visibility(), &component_id],
            )
            .await?;
        let objects = rows.into_iter().map(|row| row.get("object_id")).collect();
        Ok(objects)
    }

    pub async fn get_parent_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> EdgeResult<Option<ComponentId>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_PARENTS_FOR_COMPONENT,
                &[ctx.tenancy(), ctx.visibility(), &component_id],
            )
            .await?;
        let objects: Vec<ComponentId> = rows.into_iter().map(|row| row.get("object_id")).collect();

        Ok(if objects.is_empty() {
            None
        } else {
            // NOTE(victor) This should fail in the future, or we could auto update components for backwards compat?
            if objects.len() > 1 {
                warn!(
                    "Component({}) has more than one parent edge! Obsolete diagram data",
                    component_id
                );
            }
            Some(objects[0])
        })
    }

    pub async fn detach_component_from_parent(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> EdgeResult<ComponentId> {
        let child_comp = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(EdgeError::ComponentNotFound(component_id))?;

        let parent_id = Edge::get_parent_for_component(ctx, component_id)
            .await?
            .ok_or(EdgeError::ParentComponentNotFound(component_id))?;

        let child_edges = Edge::list_for_component(ctx, *child_comp.id()).await?;
        for mut child_edge in child_edges {
            if child_edge.head_component_id() == parent_id {
                child_edge.delete_and_propagate(ctx).await?;
            }
        }

        Ok(parent_id)
    }

    pub async fn list_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> EdgeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_COMPONENT,
                &[ctx.tenancy(), ctx.visibility(), &component_id],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    /// List [`Edges`](Self) for a given [`kind`](EdgeKind).
    pub async fn list_for_kind(ctx: &DalContext, kind: EdgeKind) -> EdgeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_KIND,
                &[ctx.tenancy(), ctx.visibility(), &kind.as_ref()],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    pub async fn delete_and_propagate(&mut self, ctx: &DalContext) -> EdgeResult<()> {
        let actor_user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => Some(*user_pk),
            _ => None,
        };

        ctx.txns()
            .await?
            .pg()
            .query(
                "SELECT * FROM edge_deletion_v1($1, $2, $3, $4)",
                &[ctx.tenancy(), ctx.visibility(), self.id(), &actor_user_pk],
            )
            .await?;

        diagram::summary_diagram::delete_edge_entry(ctx, self)
            .await
            .map_err(|e| EdgeError::SummaryDiagram(e.to_string()))?;

        if *self.kind() == EdgeKind::Symbolic {
            return Ok(());
        }

        let head_component_id = *{
            let head_node = Node::get_by_id(ctx, &self.head_node_id())
                .await?
                .ok_or(EdgeError::NodeNotFound(self.head_node_id))?;
            head_node
                .component(ctx)
                .await?
                .ok_or(EdgeError::ComponentNotFoundForNode(self.tail_node_id))?
                .id()
        };

        let tail_component_id = *{
            let tail_node = Node::get_by_id(ctx, &self.tail_node_id())
                .await?
                .ok_or(EdgeError::NodeNotFound(self.tail_node_id))?;
            tail_node
                .component(ctx)
                .await?
                .ok_or(EdgeError::ComponentNotFoundForNode(self.tail_node_id))?
                .id()
        };

        // This code assumes that every connection is established between a tail external provider and
        // a head (explicit) internal provider. That might not be the case, but it true in practice for the present state of the interface
        // (aggr frame connection to children shouldn't go through this path)
        let external_provider = {
            let socket = Socket::get_by_id(ctx, &self.tail_socket_id)
                .await?
                .ok_or(EdgeError::SocketNotFound(self.tail_socket_id))?;

            socket
                .external_provider(ctx)
                .await?
                .ok_or_else(|| EdgeError::ExternalProviderNotFoundForSocket(*socket.id()))?
        };

        let internal_provider_id = *{
            let socket = Socket::get_by_id(ctx, &self.head_socket_id())
                .await?
                .ok_or(EdgeError::SocketNotFound(self.head_socket_id))?;

            socket
                .internal_provider(ctx)
                .await?
                .ok_or_else(|| EdgeError::InternalProviderNotFoundForSocket(*socket.id()))?
                .id()
        };

        // Delete the arguments that have the same external provider of the edge, and are connected to an attribute prototype for
        let mut edge_argument = AttributePrototypeArgument::find_for_providers_and_components(
            ctx,
            external_provider.id(),
            &internal_provider_id,
            &tail_component_id,
            &head_component_id,
        )
        .await?
        .ok_or(EdgeError::AttributePrototypeNotFound)?;

        edge_argument.delete_by_id(ctx).await?;

        let read_context = AttributeReadContext {
            prop_id: Some(PropId::NONE),
            internal_provider_id: Some(internal_provider_id),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(head_component_id),
        };

        let mut attr_value = AttributeValue::find_for_context(ctx, read_context)
            .await?
            .ok_or(EdgeError::AttributeValueNotFound)?;

        ctx.txns()
            .await?
            .pg()
            .execute(
                "SELECT attribute_value_dependencies_update_v1($1, $2, $3, $4)",
                &[
                    &ctx.tenancy().workspace_pk(),
                    &ctx.visibility().change_set_pk,
                    &ctx.visibility().deleted_at,
                    &attr_value.id(),
                ],
            )
            .await?;

        attr_value.update_from_prototype_function(ctx).await?;

        ctx.enqueue_dependent_values_update(vec![*attr_value.id()])
            .await?;

        diagram::summary_diagram::delete_edge_entry(ctx, self)
            .await
            .map_err(|e| EdgeError::SummaryDiagram(e.to_string()))?;

        Ok(())
    }

    pub async fn restore_by_id(ctx: &DalContext, edge_id: EdgeId) -> EdgeResult<Option<Self>> {
        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        let deleted_edge = Edge::get_by_id(ctx_with_deleted, &edge_id)
            .await?
            .ok_or(EdgeError::EdgeNotFound(edge_id))?;

        if deleted_edge.visibility().deleted_at.is_none()
            || deleted_edge.visibility().change_set_pk != ctx.visibility().change_set_pk
        {
            return Err(EdgeError::RestoringNonDeletedEdge(edge_id));
        }

        // check if the head or tail are marked as deleted
        // if this is the case, error out here
        let head_node_id = &deleted_edge.head_node_id();
        let maybe_head_node = Node::get_by_id(ctx_with_deleted, head_node_id).await?;
        let tail_node_id = &deleted_edge.tail_node_id();
        let maybe_tail_node = Node::get_by_id(ctx_with_deleted, tail_node_id).await?;

        // we need to check if either sides of the edge are not deleted
        if let Some(head_node) = maybe_head_node {
            if head_node.visibility().change_set_pk == ctx.visibility().change_set_pk
                && head_node.visibility().deleted_at.is_some()
            {
                return Err(EdgeError::RestoringAnEdgeToDeletedNode(
                    edge_id,
                    *head_node.id(),
                ));
            }
        }

        if let Some(tail_node) = maybe_tail_node {
            if tail_node.visibility().change_set_pk == ctx.visibility().change_set_pk
                && tail_node.visibility().deleted_at.is_some()
            {
                return Err(EdgeError::RestoringAnEdgeToDeletedNode(
                    edge_id,
                    *tail_node.id(),
                ));
            }
        }

        let head_socket_id = &deleted_edge.head_socket_id();
        let tail_socket_id = &deleted_edge.tail_socket_id();

        let edge_kind = deleted_edge.kind;

        diagram::summary_diagram::restore_edge_entry(ctx, &deleted_edge)
            .await
            .map_err(|e| EdgeError::SummaryDiagram(e.to_string()))?;

        // Note(victor): We hard delete the edge on the changeset so the status calculations
        // does not think it is a newly created one (Yeah yeah I know I know)
        deleted_edge.hard_delete(ctx_with_deleted).await?;

        if edge_kind == EdgeKind::Symbolic {
            return Ok(Edge::get_by_id(ctx, &edge_id).await?);
        }

        // Restore the Attribute Prototype Argument
        let head_component_id = *{
            let head_node = Node::get_by_id(ctx_with_deleted, head_node_id)
                .await?
                .ok_or(EdgeError::NodeNotFound(*head_node_id))?;
            head_node
                .component(ctx_with_deleted)
                .await?
                .ok_or(EdgeError::ComponentNotFoundForNode(*head_node_id))?
                .id()
        };

        let tail_component_id = *{
            let tail_node = Node::get_by_id(ctx_with_deleted, tail_node_id)
                .await?
                .ok_or(EdgeError::NodeNotFound(*tail_node_id))?;
            tail_node
                .component(ctx_with_deleted)
                .await?
                .ok_or(EdgeError::ComponentNotFoundForNode(*tail_node_id))?
                .id()
        };

        // This code assumes that every connection is established between a tail external provider and
        // a head (explicit) internal provider. That might not be the case, but it true in practice for the present state of the interface
        // (aggr frame connection to children shouldn't go through this path)
        let external_provider_id = *{
            let socket = Socket::get_by_id(ctx_with_deleted, tail_socket_id)
                .await?
                .ok_or(EdgeError::SocketNotFound(*tail_socket_id))?;

            socket
                .external_provider(ctx_with_deleted)
                .await?
                .ok_or_else(|| EdgeError::ExternalProviderNotFoundForSocket(*socket.id()))?
                .id()
        };

        let internal_provider_id = *{
            let socket = Socket::get_by_id(ctx_with_deleted, head_socket_id)
                .await?
                .ok_or(EdgeError::SocketNotFound(*head_socket_id))?;

            socket
                .internal_provider(ctx_with_deleted)
                .await?
                .ok_or_else(|| EdgeError::InternalProviderNotFoundForSocket(*socket.id()))?
                .id()
        };

        let mut edge_argument = AttributePrototypeArgument::find_for_providers_and_components(
            ctx_with_deleted,
            &external_provider_id,
            &internal_provider_id,
            &tail_component_id,
            &head_component_id,
        )
        .await?
        .ok_or(EdgeError::AttributePrototypeNotFound)?;

        edge_argument.undelete(ctx_with_deleted).await?;

        // Trigger a dependent values update
        let read_context = AttributeReadContext {
            prop_id: Some(PropId::NONE),
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(external_provider_id),
            component_id: Some(tail_component_id),
        };

        let attr_value = AttributeValue::find_for_context(ctx_with_deleted, read_context)
            .await?
            .ok_or(EdgeError::AttributeValueNotFound)?;

        ctx.txns()
            .await?
            .pg()
            .execute(
                "SELECT attribute_value_dependencies_update_v1($1, $2, $3, $4)",
                &[
                    &ctx.tenancy().workspace_pk(),
                    &ctx.visibility().change_set_pk,
                    &ctx.visibility().deleted_at,
                    attr_value.id(),
                ],
            )
            .await?;

        ctx.enqueue_dependent_values_update(vec![*attr_value.id()])
            .await?;

        Ok(Edge::get_by_id(ctx, &edge_id).await?)
    }

    /// This function should be only called by [`Self::new_for_connection()`] and integration tests.
    /// The latter is why this function is public.
    ///
    /// When in the context of [`Connections`](crate::Connection), the following terminology is
    /// relevant:
    /// - _"head":_ where the connection is going to
    /// - _"tail":_ where the connection is coming from
    ///
    /// Currently this func only supports connecting via the identity [`Func`](crate::Func), refactoring
    /// is necessary to support other transformation functions for edge connections.
    pub async fn connect_providers_for_components(
        ctx: &DalContext,
        head_explicit_internal_provider_id: InternalProviderId,
        head_component_id: ComponentId,
        tail_external_provider_id: ExternalProviderId,
        tail_component_id: ComponentId,
    ) -> EdgeResult<()> {
        let head_explicit_internal_provider: InternalProvider =
            InternalProvider::get_by_id(ctx, &head_explicit_internal_provider_id)
                .await?
                .ok_or(EdgeError::InternalProviderNotFound(
                    head_explicit_internal_provider_id,
                ))?;
        let tail_external_provider: ExternalProvider =
            ExternalProvider::get_by_id(ctx, &tail_external_provider_id)
                .await?
                .ok_or(EdgeError::ExternalProviderNotFound(
                    tail_external_provider_id,
                ))?;

        // Check that the explicit internal provider is actually explicit and find its attribute
        // prototype id.
        if head_explicit_internal_provider.is_internal_consumer() {
            return Err(EdgeError::FoundImplicitInternalProvider(
                *head_explicit_internal_provider.id(),
            ));
        }

        let attr_read_context = AttributeReadContext {
            prop_id: Some(PropId::NONE),
            internal_provider_id: Some(head_explicit_internal_provider_id),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(head_component_id),
        };

        let attribute_value = AttributeValue::find_for_context(ctx, attr_read_context)
            .await?
            .ok_or(EdgeError::AttributeValueNotFound)?;

        let attribute_prototype = attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(EdgeError::AttributePrototypeNotFound)?;

        let (_identity_func, identity_func_argument) = Func::identity_with_argument(ctx).await?;

        // Now, we can create the inter component attribute prototype argument.
        AttributePrototypeArgument::new_for_inter_component(
            ctx,
            *attribute_prototype.id(),
            *identity_func_argument.id(),
            head_component_id,
            tail_component_id,
            *tail_external_provider.id(),
        )
        .await?;
        Ok(())
    }

    pub async fn connect_internal_providers_for_components(
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
        head_component_id: ComponentId,
        tail_component_id: ComponentId,
    ) -> EdgeResult<()> {
        let attr_read_context = AttributeReadContext {
            prop_id: Some(PropId::NONE),
            internal_provider_id: Some(internal_provider_id),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(head_component_id),
        };

        let attribute_value = AttributeValue::find_for_context(ctx, attr_read_context)
            .await?
            .ok_or(EdgeError::AttributeValueNotFound)?;

        let attribute_prototype = attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(EdgeError::AttributePrototypeNotFound)?;

        let (_identity_func, identity_func_argument) = Func::identity_with_argument(ctx).await?;

        // Now, we can create the inter component attribute prototype argument.
        AttributePrototypeArgument::new_explicit_internal_to_explicit_internal_inter_component(
            ctx,
            *attribute_prototype.id(),
            *identity_func_argument.id(),
            head_component_id,
            tail_component_id,
            internal_provider_id,
        )
        .await?;
        Ok(())
    }

    pub async fn connect_external_providers_for_components(
        ctx: &DalContext,
        external_provider_id: ExternalProviderId,
        head_component_id: ComponentId,
        tail_component_id: ComponentId,
    ) -> EdgeResult<()> {
        let attr_read_context = AttributeReadContext {
            prop_id: Some(PropId::NONE),
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(external_provider_id),
            component_id: Some(head_component_id),
        };

        let attribute_value = AttributeValue::find_for_context(ctx, attr_read_context)
            .await?
            .ok_or(EdgeError::AttributeValueNotFound)?;

        let attribute_prototype = attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(EdgeError::AttributePrototypeNotFound)?;

        let (_identity_func, identity_func_argument) = Func::identity_with_argument(ctx).await?;

        // Now, we can create the inter component attribute prototype argument.
        AttributePrototypeArgument::new_external_to_external_inter_component(
            ctx,
            *attribute_prototype.id(),
            *identity_func_argument.id(),
            head_component_id,
            tail_component_id,
            external_provider_id,
        )
        .await?;
        Ok(())
    }

    pub fn head_component_id(&self) -> ComponentId {
        self.head_object_id().into()
    }

    pub fn tail_component_id(&self) -> ComponentId {
        self.tail_object_id().into()
    }
}
