use content_store::ContentHash;
use rand::prelude::SliceRandom;

use serde::{Deserialize, Serialize};



use strum::EnumDiscriminants;
use telemetry::prelude::*;



use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{
    pk, StandardModel, Timestamp,
};


// const LIST_FOR_KIND: &str = include_str!("queries/node/list_for_kind.sql");
// const LIST_LIVE: &str = include_str!("queries/node/list_live.sql");

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

/// Visual representation of a [`Component`](crate::Component) for a given [`kind`](NodeKind).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    id: NodeId,
    #[serde(flatten)]
    timestamp: Timestamp,
    kind: NodeKind,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct NodeGraphNode {
    id: NodeId,
    content_address: ContentAddress,
    content: NodeContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum NodeContent {
    V1(NodeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NodeContentV1 {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    pub kind: NodeKind,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

impl NodeGraphNode {
    pub fn assemble(
        id: impl Into<NodeId>,
        content_hash: ContentHash,
        content: NodeContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::Node(content_hash),
            content,
        }
    }
}

impl Node {
    pub fn assemble(id: NodeId, inner: &NodeContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            kind: inner.kind,
            x: inner.x.clone(),
            y: inner.y.clone(),
            width: inner.width.clone(),
            height: inner.height.clone(),
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }
}

impl From<Node> for NodeContentV1 {
    fn from(value: Node) -> Self {
        Self {
            timestamp: value.timestamp,
            kind: value.kind,
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

// impl Node {
//     /// List all "live" [`Nodes`](Node) for a given [`NodeKind`](NodeKind).
//     ///
//     /// The [`DalContext`](crate::DalContext) should be provided with "deletion"
//     /// [`Visibility`](crate::Visibility).
//     pub async fn list_live(ctx: &DalContext, kind: NodeKind) -> NodeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_LIVE,
//                 &[
//                     ctx.tenancy(),
//                     &ctx.visibility().to_deleted(),
//                     &kind.as_ref(),
//                 ],
//             )
//             .await?;
//         Ok(objects_from_rows(rows)?)
//     }

//     /// Find all [`NodeIds`](Self) for a given [`NodeKind`].
//     #[instrument(skip_all)]
//     pub async fn list_for_kind(ctx: &DalContext, kind: NodeKind) -> NodeResult<HashSet<NodeId>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_KIND,
//                 &[ctx.tenancy(), ctx.visibility(), &kind.as_ref()],
//             )
//             .await?;
//         let mut node_ids = HashSet::new();
//         for row in rows {
//             let node_id: NodeId = row.try_get("node_id")?;
//             node_ids.insert(node_id);
//         }
//         Ok(node_ids)
//     }

//     /// List all [`Nodes`](Self) of kind [`configuration`](NodeKind::Configuration) in
//     /// [`topological`](https://en.wikipedia.org/wiki/Topological_sorting) order. The order will
//     /// be also be stable.
//     pub async fn list_topologically_sorted_configuration_nodes_with_stable_ordering(
//         ctx: &DalContext,
//         shuffle_edges: bool,
//     ) -> NodeResult<Vec<NodeId>> {
//         let total_start = std::time::Instant::now();
//         let ctx_with_deleted = &ctx.clone_with_delete_visibility();

//         // Gather all nodes with at least one edge.
//         let mut edges = Edge::list_for_kind(ctx_with_deleted, EdgeKind::Configuration)
//             .await
//             .map_err(|e| NodeError::Edge(e.to_string()))?;
//         if shuffle_edges {
//             edges.shuffle(&mut thread_rng());
//         }

//         // Populate the nodes map based on all configuration edges. The "key" is every node with at
//         // least one edge. The "value" is a set of nodes that the "key" node depends on (i.e. the
//         // set of nodes are sources/tails in edges and the "key" node is the destination/head in
//         // in edges).
//         let mut nodes: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
//         for edge in edges {
//             nodes
//                 .entry(edge.head_node_id())
//                 .and_modify(|set| {
//                     set.insert(edge.tail_node_id());
//                 })
//                 .or_insert_with(|| {
//                     let mut set = HashSet::new();
//                     set.insert(edge.tail_node_id());
//                     set
//                 });
//         }

//         // Add all floating nodes (those without edges).
//         for potential_floating_node in
//             Self::list_for_kind(ctx_with_deleted, NodeKind::Configuration).await?
//         {
//             if nodes.get(&potential_floating_node).is_none() {
//                 nodes.insert(potential_floating_node, HashSet::new());
//             }
//         }

//         // Gather all results based on the nodes and their "depends_on" sets. This is a topological
//         // sort with stable ordering.
//         let mut results = Vec::new();
//         loop {
//             let mut siblings: Vec<NodeId> = Vec::new();

//             // For each node in the map, find siblings (those whose "depends_on" sets are empty)
//             for (node, depends_on) in &mut nodes {
//                 if depends_on.is_empty() {
//                     siblings.push(*node);
//                 }
//             }

//             // If we found no siblings, then we have processed every node in the map and are ready
//             // to exit the loop.
//             if siblings.is_empty() {
//                 break;
//             }

//             // Remove each sibling from the map's "keys".
//             for sibling in &siblings {
//                 nodes.remove(sibling);
//             }

//             // Remove each sibling from the map's "values".
//             nodes.iter_mut().for_each(|(_, depends_on)| {
//                 for sibling in &siblings {
//                     depends_on.remove(sibling);
//                 }
//             });

//             // Provide stable ordering by sorting the siblings before extending the results.
//             siblings.sort();
//             results.extend(siblings);
//         }

//         debug!(
//             "listing topologically sorted configuration nodes with stable ordering took {:?}",
//             total_start.elapsed()
//         );
//         Ok(results)
//     }
// }
