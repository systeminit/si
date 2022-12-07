//! This module contains [`Edge`], the mathematical "edge" between two [`Nodes`](crate::Node) in a
//! graph.

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::node::NodeId;
use crate::{
    impl_standard_model, pk, socket::SocketId, standard_model, standard_model_accessor,
    ComponentId, ExternalProviderError, Func, HistoryEventError, InternalProviderError,
    ReadTenancyError, StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
};
use crate::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, Component, DalContext,
    ExternalProvider, ExternalProviderId, InternalProvider, InternalProviderId, NodeError,
};

const LIST_PARENTS_FOR_COMPONENT: &str =
    include_str!("queries/edge_list_parents_for_component.sql");

#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),

    #[error("component error: {0}")]
    Component(String),
    #[error("external provider not found for id: {0}")]
    ExternalProviderNotFound(ExternalProviderId),
    #[error("external provider not found for socket id: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("implicit internal provider cannot be used for inter component connection: {0}")]
    FoundImplicitInternalProvider(InternalProviderId),
    #[error("internal provider not found for id: {0}")]
    InternalProviderNotFound(InternalProviderId),
    #[error("internal provider not found for socket id: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error("cannot find identity func")]
    IdentityFuncNotFound,
    #[error("cannot find identity func argument")]
    IdentityFuncArgNotFound,
}

pub type EdgeResult<T> = Result<T, EdgeError>;

/// Used to dictate what [`EdgeKinds`](EdgeKind) can be for the head and tail of an [`Edges`](Edge).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum VertexObjectKind {
    /// Used for [`Nodes`](crate::Node) of [`NodeKind::Configuration`](crate::NodeKind::Configuration).
    Configuration,
}

/// The kind of an [`Edge`](Edge). This provides the ability to categorize [`Edges`](Edge)
/// and create [`EdgeKind`](Self)-specific graphs.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EdgeKind {
    /// Used to connect a configuration to another configuration.
    Configuration,
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
    #[serde(flatten)]
    tenancy: WriteTenancy,
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
        Self::from(id.0)
    }
}

impl From<ComponentId> for EdgeObjectId {
    fn from(id: ComponentId) -> Self {
        Self::from(ulid::Ulid::from(id))
    }
}

impl Edge {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
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
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM edge_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    ctx.write_tenancy(),
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
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
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
    #[instrument(skip_all)]
    pub async fn new_for_connection(
        ctx: &DalContext,
        head_node_id: NodeId,
        head_socket_id: SocketId,
        tail_node_id: NodeId,
        tail_socket_id: SocketId,
    ) -> EdgeResult<Self> {
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

        // TODO(nick): allow for more transformation functions.
        Self::connect_providers_for_components(
            ctx,
            *head_explicit_internal_provider.id(),
            *head_component.id(),
            *tail_external_provider.id(),
            *tail_component.id(),
        )
        .await?;

        // NOTE(nick): a lot of hardcoded values here that'll likely need to be adjusted.
        let edge = Edge::new(
            ctx,
            EdgeKind::Configuration,
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

    pub async fn list_parents_for_component(
        ctx: &DalContext,
        head_component_id: ComponentId,
    ) -> EdgeResult<Vec<ComponentId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_PARENTS_FOR_COMPONENT,
                &[ctx.read_tenancy(), ctx.visibility(), &head_component_id],
            )
            .await?;
        let objects = rows
            .into_iter()
            .map(|row| row.get("tail_object_id"))
            .collect();
        Ok(objects)
    }

    /// This function should be only called by [`Self::new_for_connection()`] and integration tests.
    /// The latter is why this function is public.
    ///
    /// When in the context of [`Connections`](crate::Connection), the following terminology is
    /// relevant:
    /// - _"head":_ where the connection is going to
    /// - _"tail":_ where the connection is coming from
    ///
    /// Currently this func only supports connecting via the "si:identity" func, refactoring
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
        let head_explicit_internal_provider_attribute_prototype_id =
            head_explicit_internal_provider
                .attribute_prototype_id()
                .ok_or(InternalProviderError::EmptyAttributePrototype)?;

        let identity_func = Func::find_by_attr(ctx, "name", &"si:identity")
            .await?
            .pop()
            .ok_or(EdgeError::IdentityFuncNotFound)?;
        let identity_func_arg =
            FuncArgument::find_by_name_for_func(ctx, "identity", *identity_func.id())
                .await?
                .ok_or(EdgeError::IdentityFuncArgNotFound)?;

        // Now, we can create the inter component attribute prototype argument.
        AttributePrototypeArgument::new_for_inter_component(
            ctx,
            *head_explicit_internal_provider_attribute_prototype_id,
            *identity_func_arg.id(),
            head_component_id,
            tail_component_id,
            *tail_external_provider.id(),
        )
        .await?;
        Ok(())
    }
}
