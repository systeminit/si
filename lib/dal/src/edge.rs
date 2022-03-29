use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::node::NodeId;
use crate::{
    impl_standard_model, pk, socket::SocketId, standard_model, standard_model_accessor,
    ComponentId, HistoryEventError, ReadTenancyError, StandardModel, StandardModelError, SystemId,
    Timestamp, Visibility, WriteTenancy,
};

const FIND_PARENT_COMPONENTS: &str = include_str!("./queries/edge_find_parent_components.sql");

#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("standard model error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
}

pub type EdgeResult<T> = Result<T, EdgeError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum VertexObjectKind {
    Component,
    System,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EdgeKind {
    Configures,
    Includes,
    Deployment,
    Component,
    Implementation,
}

pk!(EdgeId);
pk!(EdgePk);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pk: EdgePk,
    id: EdgeId,
    kind: EdgeKind,
    // NOTE: Would love to flatten this, but serde doesn't allow flatten and rename.
    head_node_id: NodeId,
    head_object_kind: VertexObjectKind,
    head_object_id: i64,
    head_socket_id: SocketId,
    tail_node_id: NodeId,
    tail_object_kind: VertexObjectKind,
    tail_object_id: i64,
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

impl Edge {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        kind: EdgeKind,
        head_node_id: NodeId,
        head_object_kind: VertexObjectKind,
        head_object_id: i64,
        head_socket_id: SocketId,
        tail_node_id: NodeId,
        tail_object_kind: VertexObjectKind,
        tail_object_id: i64,
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

    standard_model_accessor!(kind, Enum(EdgeKind), EdgeResult);

    // Sockets
    standard_model_accessor!(head_node_id, Pk(NodeId), EdgeResult);
    standard_model_accessor!(head_object_kind, Enum(VertexObjectKind), EdgeResult);
    standard_model_accessor!(head_object_id, i64, EdgeResult);
    standard_model_accessor!(head_socket_id, Pk(SocketId), EdgeResult);
    standard_model_accessor!(tail_node_id, Pk(NodeId), EdgeResult);
    standard_model_accessor!(tail_object_kind, Enum(VertexObjectKind), EdgeResult);
    standard_model_accessor!(tail_object_id, i64, EdgeResult);
    standard_model_accessor!(tail_socket_id, Pk(SocketId), EdgeResult);

    pub async fn find_component_configuration_parents(
        ctx: &DalContext<'_, '_>,
        component_id: &ComponentId,
    ) -> EdgeResult<Vec<ComponentId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_PARENT_COMPONENTS,
                &[ctx.read_tenancy(), ctx.visibility(), &component_id],
            )
            .await?;
        let objects = rows
            .into_iter()
            .map(|row| row.get("tail_object_id"))
            .collect();
        Ok(objects)
    }

    pub async fn include_component_in_system(
        ctx: &DalContext<'_, '_>,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> EdgeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM edge_include_component_in_system_v1($1, $2, $3, $4)",
                &[
                    &ctx.read_tenancy(),
                    ctx.visibility(),
                    component_id,
                    system_id,
                ],
            )
            .await?;

        let object = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    pub async fn include_component_in_node(
        ctx: &DalContext<'_, '_>,
        component_id: &ComponentId,
        parent_node_id: &NodeId,
    ) -> EdgeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM edge_include_component_in_node_v1($1, $2, $3, $4)",
                &[
                    &ctx.read_tenancy(),
                    ctx.visibility(),
                    component_id,
                    parent_node_id,
                ],
            )
            .await?;

        let object = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }
}
