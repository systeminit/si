use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::node::NodeId;
use crate::{
    impl_standard_model, pk, socket::SocketId, standard_model, standard_model_accessor,
    ComponentId, HistoryActor, HistoryEventError, ReadTenancy, ReadTenancyError, StandardModel,
    StandardModelError, SystemId, Tenancy, Timestamp, Visibility, WriteTenancy,
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

impl Edge {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
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
        let row = txn
            .query_one(
                "SELECT object FROM edge_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    write_tenancy,
                    &visibility,
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
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            &write_tenancy.into(),
            visibility,
            history_actor,
            row,
        )
        .await?;
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
        txn: &PgTxn<'_>,
        read_tenancy: &ReadTenancy,
        visibility: &Visibility,
        component_id: &ComponentId,
    ) -> EdgeResult<Vec<ComponentId>> {
        let rows = txn
            .query(
                FIND_PARENT_COMPONENTS,
                &[read_tenancy, &visibility, &component_id],
            )
            .await?;
        let objects = rows
            .into_iter()
            .map(|row| row.get("tail_object_id"))
            .collect();
        Ok(objects)
    }

    pub async fn include_component_in_system(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> EdgeResult<Self> {
        let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;

        let row = txn
            .query_one(
                "SELECT object FROM edge_include_component_in_system_v1($1, $2, $3, $4)",
                &[&read_tenancy, &visibility, component_id, system_id],
            )
            .await?;

        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            &(&read_tenancy).into(),
            visibility,
            history_actor,
            row,
        )
        .await?;

        Ok(object)
    }

    pub async fn include_component_in_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        component_id: &ComponentId,
        parent_node_id: &NodeId,
    ) -> EdgeResult<Self> {
        let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;

        let row = txn
            .query_one(
                "SELECT object FROM edge_include_component_in_node_v1($1, $2, $3, $4)",
                &[&read_tenancy, &visibility, component_id, parent_node_id],
            )
            .await?;

        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            &(&read_tenancy).into(),
            visibility,
            history_actor,
            row,
        )
        .await?;

        Ok(object)
    }
}
