use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, node::NodeId, pk, standard_model, standard_model_accessor,
    standard_model_belongs_to, DiagramKind, HistoryEventError, Node, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum NodePositionError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
}

const LIST_FOR_NODE: &str = include_str!("queries/node_position_list_for_node.sql");

pub type NodePositionResult<T> = Result<T, NodePositionError>;

pk!(NodePositionPk);
pk!(NodePositionId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NodePosition {
    pk: NodePositionPk,
    id: NodePositionId,
    diagram_kind: DiagramKind,
    x: String,
    y: String,
    width: Option<String>,
    height: Option<String>,
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
    /// Creates a new [`NodePosition`](Self) and sets its "belongs_to_id" to the provided
    /// [`NodeId`](crate::Node).
    pub async fn new(
        ctx: &DalContext,
        node_id: NodeId,
        diagram_kind: DiagramKind,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
        width: Option<impl AsRef<str>>,
        height: Option<impl AsRef<str>>,
    ) -> NodePositionResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM node_position_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &diagram_kind.as_ref(),
                    &x.as_ref(),
                    &y.as_ref(),
                    &width.as_ref().map(|w| w.as_ref()),
                    &height.as_ref().map(|h| h.as_ref()),
                ],
            )
            .await?;
        let node_position: Self = standard_model::finish_create_from_row(ctx, row).await?;
        node_position.set_node(ctx, &node_id).await?;
        Ok(node_position)
    }

    pub async fn list_for_node(ctx: &DalContext, node_id: NodeId) -> NodePositionResult<Vec<Self>> {
        let rows = ctx
            .pg_txn()
            .query(LIST_FOR_NODE, &[ctx.tenancy(), ctx.visibility(), &node_id])
            .await?;
        let objects = standard_model::objects_from_rows(rows)?;
        Ok(objects)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_by_node_id(
        ctx: &DalContext,
        diagram_kind: DiagramKind,
        node_id: NodeId,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
        width: Option<impl AsRef<str>>,
        height: Option<impl AsRef<str>>,
    ) -> NodePositionResult<Self> {
        for mut position in Self::list_for_node(ctx, node_id).await? {
            // Modify and return the position if found.
            if position.diagram_kind == diagram_kind {
                position.set_x(ctx, x.as_ref()).await?;
                position.set_y(ctx, y.as_ref()).await?;
                position
                    .set_width(ctx, width.as_ref().map(|val| val.as_ref()))
                    .await?;
                position
                    .set_height(ctx, height.as_ref().map(|val| val.as_ref()))
                    .await?;
                return Ok(position);
            }
        }

        let obj = Self::new(ctx, node_id, diagram_kind, x, y, width, height).await?;
        Ok(obj)
    }

    standard_model_accessor!(diagram_kind, Enum(DiagramKind), NodePositionResult);
    standard_model_accessor!(x, String, NodePositionResult);
    standard_model_accessor!(y, String, NodePositionResult);
    standard_model_accessor!(width, Option<String>, NodePositionResult);
    standard_model_accessor!(height, Option<String>, NodePositionResult);

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
