use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, node::NodeId, pk, standard_model, standard_model_accessor,
    standard_model_belongs_to, HistoryEventError, Node, ReadTenancyError, SchematicKind,
    StandardModel, StandardModelError, SystemId, Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum NodePositionError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
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
    deployment_node_id: Option<NodeId>,
    x: String,
    y: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
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
        ctx: &DalContext<'_, '_>,
        schematic_kind: SchematicKind,
        root_node_id: NodeId,
        system_id: Option<SystemId>,
        deployment_node_id: Option<NodeId>,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
    ) -> NodePositionResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM node_position_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &schematic_kind.as_ref(),
                    &root_node_id,
                    &system_id,
                    &deployment_node_id,
                    &x.as_ref(),
                    &y.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    pub async fn find_by_node_id(
        ctx: &DalContext<'_, '_>,
        system_id: Option<SystemId>,
        root_node_id: NodeId,
        node_id: NodeId,
    ) -> NodePositionResult<Vec<Self>> {
        let rows = ctx
            .pg_txn()
            .query(
                FIND_NODE_POSITION_BY_NODE_ID,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &system_id,
                    &root_node_id,
                    &node_id,
                ],
            )
            .await?;
        let objects = standard_model::objects_from_rows(rows)?;
        Ok(objects)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_by_node_id(
        ctx: &DalContext<'_, '_>,
        schematic_kind: SchematicKind,
        system_id: Option<SystemId>,
        deployment_node_id: Option<NodeId>,
        root_node_id: NodeId,
        node_id: NodeId,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
    ) -> NodePositionResult<Self> {
        for mut position in Self::find_by_node_id(ctx, system_id, root_node_id, node_id).await? {
            if position.deployment_node_id == deployment_node_id
                && position.schematic_kind == schematic_kind
            {
                position.set_x(ctx, x.as_ref()).await?;
                position.set_y(ctx, y.as_ref()).await?;
                return Ok(position);
            }
        }
        let obj = Self::new(
            ctx,
            schematic_kind,
            root_node_id,
            system_id,
            deployment_node_id,
            x,
            y,
        )
        .await?;
        obj.set_node(ctx, &node_id).await?;
        Ok(obj)
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
