use rand::prelude::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use telemetry::prelude::*;
use thiserror::Error;

use crate::edge::EdgeKind;
use crate::{
    impl_standard_model, pk, schema::variant::SchemaVariantError, standard_model,
    standard_model_accessor, standard_model_belongs_to, Component, ComponentId, HistoryEventError,
    StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
};
use crate::{DalContext, Edge, SchemaError};

const LIST_FOR_KIND: &str = include_str!("queries/node/list_for_kind.sql");
const LIST_CONNECTED_FOR_KIND: &str = include_str!("queries/node/list_connected_for_kind.sql");

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("cannot find schema id to generate node template")]
    SchemaIdNotFound,
    #[error("cannot generate node template with missing default schema variant")]
    SchemaMissingDefaultVariant,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("component is None")]
    ComponentIsNone,
    #[error("could not find node with ID: {0}")]
    NotFound(NodeId),
    #[error("edge error: {0}")]
    Edge(String),
}

pub type NodeResult<T> = Result<T, NodeError>;

pk!(NodePk);
pk!(NodeId);

/// The kind of a given [`Node`](Node) that corresponds to the [`DiagramKind`](crate::DiagramKind).
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
    strum_macros::EnumIter,
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
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
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
    #[instrument(skip_all)]
    pub async fn new(ctx: &DalContext, kind: &NodeKind) -> NodeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM node_create_v1($1, $2, $3)",
                &[ctx.write_tenancy(), ctx.visibility(), &kind.as_ref()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(kind, Enum(NodeKind), NodeResult);

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

    /// Find all [`NodeIds`](Self) for a given [`NodeKind`].
    #[instrument(skip_all)]
    pub async fn list_for_kind(ctx: &DalContext, kind: NodeKind) -> NodeResult<HashSet<NodeId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_KIND,
                &[ctx.write_tenancy(), ctx.visibility(), &kind.as_ref()],
            )
            .await?;
        let mut node_ids = HashSet::new();
        for row in rows {
            let node_id: NodeId = row.try_get("node_id")?;
            node_ids.insert(node_id);
        }
        Ok(node_ids)
    }

    /// Find all [`NodeIds`](Self) for a given [`NodeKind`] that are connected to at least one
    /// [`Edge`](crate::Edge).
    #[instrument(skip_all)]
    pub async fn list_connected_for_kind(
        ctx: &DalContext,
        kind: NodeKind,
    ) -> NodeResult<HashSet<NodeId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_CONNECTED_FOR_KIND,
                &[ctx.write_tenancy(), ctx.visibility(), &kind.as_ref()],
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
    /// [`topological-ish`](https://en.wikipedia.org/wiki/Topological_sorting) order.
    pub async fn list_topologically_ish_sorted_configuration_nodes(
        ctx: &DalContext,
        shuffle_edges: bool,
    ) -> NodeResult<Vec<NodeId>> {
        let total_start = std::time::Instant::now();

        // Gather all the edges, nodes, and connected nodes. There are likely many nodes that are
        // connected, so there's data duplication here that can be fixed in the future. In addition,
        // we optionally shuffle edges if a test requires it.
        let mut edges = Edge::list_for_kind(ctx, EdgeKind::Configuration)
            .await
            .map_err(|e| NodeError::Edge(e.to_string()))?;
        if shuffle_edges {
            edges.shuffle(&mut thread_rng());
        }
        let all_nodes = Self::list_for_kind(ctx, NodeKind::Configuration).await?;
        let all_connected_nodes =
            Self::list_connected_for_kind(ctx, NodeKind::Configuration).await?;

        // We must track destinations because all nodes are sources, but are not destinations.
        //
        // Runtime complexity: O(n) where "n" is an edge
        let mut nodes_with_immediate_destinations: HashMap<NodeId, HashSet<NodeId>> =
            HashMap::new();
        for edge in edges {
            match nodes_with_immediate_destinations.entry(edge.tail_node_id()) {
                Entry::Occupied(entry) => {
                    entry.into_mut().insert(edge.head_node_id());
                }
                Entry::Vacant(entry) => {
                    entry.insert({
                        let mut set = HashSet::new();
                        set.insert(edge.head_node_id());
                        set
                    });
                }
            }
        }

        // Add the full destination lineage for every node.
        //
        // Runtime complexity: O(n*m) where "n" is a node and "m" is a full lineage node (i.e. a
        // "head" of "n" or a "head of a head, etc." of "n")
        let mut nodes_with_full_destination_lineage: HashMap<NodeId, HashSet<NodeId>> =
            HashMap::new();
        for (node_id, destinations) in &nodes_with_immediate_destinations {
            let mut full_destination_lineage = HashSet::new();
            let mut destinations_work_queue: VecDeque<NodeId> = VecDeque::new();
            destinations_work_queue.extend(destinations.clone());

            while let Some(destination) = destinations_work_queue.pop_front() {
                full_destination_lineage.insert(destination);
                if let Some(nested_destinations) =
                    nodes_with_immediate_destinations.get(&destination)
                {
                    destinations_work_queue.extend(nested_destinations);
                }
            }

            nodes_with_full_destination_lineage.insert(*node_id, full_destination_lineage);
        }

        // Sort all the node ids based on their full lineages. Insert immediately when we find the
        // first destination and continue the outer loop.
        //
        // Runtime complexity: O(n*log(n)) where "n" is a node (this is loglinear because we
        // immediately check for "floating" nodes and we insert at the first destination
        // encountered)
        let mut sorted: Vec<NodeId> = Vec::new();
        'outer: for (node_id, destinations) in &nodes_with_full_destination_lineage {
            if destinations.is_empty() {
                sorted.insert(0, *node_id);
            }

            for (index, sorted_node_id) in sorted.iter().enumerate() {
                if destinations.contains(sorted_node_id) {
                    sorted.insert(index, *node_id);
                    continue 'outer;
                }
            }

            // If no destinations were found, push to the back.
            sorted.push(*node_id);
        }

        // Finally, add all nodes that do not have edges to the front (i.e. "floating" nodes) and
        // push all connected nodes that are not tails for other edges (i.e. "terminating" nodes in
        // the directed acyclic graph) to the back.
        //
        // Runtime complexity: O(n) where "n" is a node
        for node_id in all_nodes {
            if !all_connected_nodes.contains(&node_id) {
                sorted.insert(0, node_id);
            } else if !nodes_with_full_destination_lineage.contains_key(&node_id) {
                sorted.push(node_id);
            }
        }

        debug!(
            "listing topologically-ish sorted configuration nodes took {:?}",
            total_start.elapsed()
        );
        Ok(sorted)
    }
}
