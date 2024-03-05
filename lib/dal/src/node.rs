use crate::ws_event::{WsEvent, WsEventError};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::collections::{HashMap, HashSet};
use telemetry::prelude::*;
use thiserror::Error;

use crate::edge::EdgeKind;
use crate::standard_model::objects_from_rows;
use crate::{
    diagram, impl_standard_model, pk, schema::variant::SchemaVariantError, standard_model,
    standard_model_accessor, standard_model_belongs_to, Component, ComponentId, HistoryEventError,
    StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};
use crate::{DalContext, Edge, SchemaError, TransactionsError};

const LIST_FOR_KIND: &str = include_str!("queries/node/list_for_kind.sql");
const LIST_LIVE: &str = include_str!("queries/node/list_live.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum NodeError {
    #[error("component is None")]
    ComponentIsNone,
    #[error("edge error: {0}")]
    Edge(String),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("could not find node with ID: {0}")]
    NotFound(NodeId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("cannot find schema id to generate node template")]
    SchemaIdNotFound,
    #[error("cannot generate node template with missing default schema variant")]
    SchemaMissingDefaultVariant,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("summary diagram update error: {0}")]
    SummaryDiagram(String),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type NodeResult<T> = Result<T, NodeError>;

pk!(NodePk);
pk!(NodeId);

/// The kind of a given [`Node`](Node) that corresponds to the [`DiagramKind`](crate::DiagramKind).
#[remain::sorted]
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    strum::AsRefStr,
    strum::EnumIter,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum NodeKind {
    /// The [`Node`](Node) used within [`configuration`](crate::DiagramKind::Configuration)
    /// diagrams.
    Configuration,
}

/// A mathematical node that can be used to create [`Edges`](crate::Edge).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pk: NodePk,
    id: NodeId,
    kind: NodeKind,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
    x: String,
    y: String,
    width: Option<String>,
    height: Option<String>,
}

impl_standard_model! {
    model: Node,
    pk: NodePk,
    id: NodeId,
    table_name: "nodes",
    history_event_label_base: "node",
    history_event_message_name: "Node"
}

impl Node {
    pub async fn new(ctx: &DalContext, kind: &NodeKind) -> NodeResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM node_create_v1($1, $2, $3)",
                &[ctx.tenancy(), ctx.visibility(), &kind.as_ref()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row)
            .await
            .map_err(|e| NodeError::SummaryDiagram(e.to_string()))?;
        Ok(object)
    }

    standard_model_accessor!(kind, Enum(NodeKind), NodeResult);
    standard_model_accessor!(x, String, NodeResult);
    standard_model_accessor!(y, String, NodeResult);
    standard_model_accessor!(width, Option<String>, NodeResult);
    standard_model_accessor!(height, Option<String>, NodeResult);

    standard_model_belongs_to!(
        lookup_fn: component,
        set_fn: set_component,
        unset_fn: unset_component,
        table: "node_belongs_to_component",
        model_table: "components",
        belongs_to_id: ComponentId,
        returns: Component,
        result: NodeResult,
    );

    /// List all "live" [`Nodes`](Node) for a given [`NodeKind`](NodeKind).
    ///
    /// The [`DalContext`](crate::DalContext) should be provided with "deletion"
    /// [`Visibility`](crate::Visibility).
    pub async fn list_live(ctx: &DalContext, kind: NodeKind) -> NodeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_LIVE,
                &[
                    ctx.tenancy(),
                    &ctx.visibility().to_deleted(),
                    &kind.as_ref(),
                ],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    /// Find all [`NodeIds`](Self) for a given [`NodeKind`].
    pub async fn list_for_kind(ctx: &DalContext, kind: NodeKind) -> NodeResult<HashSet<NodeId>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_KIND,
                &[ctx.tenancy(), ctx.visibility(), &kind.as_ref()],
            )
            .await?;
        let mut node_ids = HashSet::new();
        for row in rows {
            let node_id: NodeId = row.try_get("node_id")?;
            node_ids.insert(node_id);
        }
        Ok(node_ids)
    }

    /// List all [`Nodes`](Self) of kind [`configuration`](NodeKind::Configuration) in
    /// [`topological`](https://en.wikipedia.org/wiki/Topological_sorting) order. The order will
    /// be also be stable.
    pub async fn list_topologically_sorted_configuration_nodes_with_stable_ordering(
        ctx: &DalContext,
        shuffle_edges: bool,
    ) -> NodeResult<Vec<NodeId>> {
        let total_start = std::time::Instant::now();

        let mut nodes = Self::build_graph(ctx, shuffle_edges).await?;

        // Gather all results based on the nodes and their "depends_on" sets. This is a topological
        // sort with stable ordering.
        let mut results = Vec::new();
        loop {
            let mut siblings: Vec<NodeId> = Vec::new();

            // For each node in the map, find siblings (those whose "depends_on" sets are empty)
            for (node, depends_on) in &mut nodes {
                if depends_on.is_empty() {
                    siblings.push(*node);
                }
            }

            // If we found no siblings, then we have processed every node in the map and are ready
            // to exit the loop.
            if siblings.is_empty() {
                break;
            }

            // Remove each sibling from the map's "keys".
            for sibling in &siblings {
                nodes.remove(sibling);
            }

            // Remove each sibling from the map's "values".
            nodes.iter_mut().for_each(|(_, depends_on)| {
                for sibling in &siblings {
                    depends_on.remove(sibling);
                }
            });

            // Provide stable ordering by sorting the siblings before extending the results.
            siblings.sort();
            results.extend(siblings);
        }

        debug!(
            "listing topologically sorted configuration nodes with stable ordering took {:?}",
            total_start.elapsed()
        );
        Ok(results)
    }

    pub async fn set_geometry(
        &mut self,
        ctx: &DalContext,
        x: impl AsRef<str>,
        y: impl AsRef<str>,
        width: Option<impl AsRef<str>>,
        height: Option<impl AsRef<str>>,
    ) -> NodeResult<()> {
        self.set_x(ctx, x.as_ref()).await?;
        self.set_y(ctx, y.as_ref()).await?;
        self.set_width(ctx, width.as_ref().map(|val| val.as_ref()))
            .await?;
        self.set_height(ctx, height.as_ref().map(|val| val.as_ref()))
            .await?;

        diagram::summary_diagram::component_update_geometry(
            ctx,
            self.id(),
            x.as_ref(),
            y.as_ref(),
            width.as_ref(),
            height.as_ref(),
        )
        .await
        .map_err(|e| NodeError::SummaryDiagram(e.to_string()))?;
        let component = self
            .component(ctx)
            .await?
            .ok_or(NodeError::ComponentIsNone)?;

        let component_id = component.id();

        WsEvent::component_updated(ctx, *component_id)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }
}
