use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, node::NodeId, pk, standard_model, standard_model_accessor,
    standard_model_belongs_to, HistoryActor, HistoryEventError, Node, SchematicKind, StandardModel,
    StandardModelError, SystemId, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum NodePositionError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

const FIND_NODE_POSITION_BY_NODE_ID: &str =
    include_str!("./queries/node_position_find_by_node_id.sql");

pub type NodePositionResult<T> = Result<T, NodePositionError>;

pk!(NodePositionPk);
pk!(NodePositionId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NodePosition {
    pk: NodePositionPk,
    id: NodePositionId,
    schematic_kind: SchematicKind,
    root_node_id: NodeId,
    system_id: Option<SystemId>,
    x: String,
    y: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: NodePosition,
    pk: NodePositionPk,
    id: NodePositionId,
    table_name: "node_positions",
    history_event_label_base: "node_position",
    history_event_message_name: "NodePosition"
}

impl NodePosition {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        schematic_kind: SchematicKind,
        root_node_id: NodeId,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
    ) -> NodePositionResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM node_position_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    tenancy,
                    visibility,
                    &schematic_kind.as_ref(),
                    &root_node_id,
                    &x.as_ref(),
                    &y.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;

        Ok(object)
    }

    pub async fn find_by_node_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        schematic_kind: SchematicKind,
        system_id: &Option<SystemId>,
        root_node_id: NodeId,
        node_id: NodeId,
    ) -> NodePositionResult<Option<Self>> {
        let row = txn
            .query_opt(
                FIND_NODE_POSITION_BY_NODE_ID,
                &[
                    tenancy,
                    visibility,
                    &schematic_kind.as_ref(),
                    &system_id,
                    &root_node_id,
                    &node_id,
                ],
            )
            .await?;
        let object = standard_model::option_object_from_row(row)?;
        Ok(object)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_by_node_id(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        schematic_kind: SchematicKind,
        system_id: &Option<SystemId>,
        root_node_id: NodeId,
        node_id: NodeId,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
    ) -> NodePositionResult<Self> {
        if let Some(mut position) = Self::find_by_node_id(
            txn,
            tenancy,
            visibility,
            schematic_kind,
            system_id,
            root_node_id,
            node_id,
        )
        .await?
        {
            position
                .set_x(txn, nats, visibility, history_actor, x.as_ref())
                .await?;
            position
                .set_y(txn, nats, visibility, history_actor, y.as_ref())
                .await?;
            Ok(position)
        } else {
            let mut obj = Self::new(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                schematic_kind,
                root_node_id,
                x,
                y,
            )
            .await?;
            if let Some(system_id) = system_id {
                obj.set_system_id(txn, nats, visibility, history_actor, Some(*system_id))
                    .await?;
            }
            obj.set_node(txn, nats, visibility, history_actor, &node_id)
                .await?;
            Ok(obj)
        }
    }

    standard_model_accessor!(schematic_kind, Enum(SchematicKind), NodePositionResult);
    standard_model_accessor!(root_node_id, Pk(NodeId), NodePositionResult);
    standard_model_accessor!(system_id, OptionBigInt<SystemId>, NodePositionResult);
    standard_model_accessor!(x, String, NodePositionResult);
    standard_model_accessor!(y, String, NodePositionResult);

    standard_model_belongs_to!(
        lookup_fn: node,
        set_fn: set_node,
        unset_fn: unset_node,
        table: "node_position_belongs_to_node",
        model_table: "nodes",
        belongs_to_id: NodeId,
        returns: Node,
        result: NodePositionResult,
    );
}
