use content_store::{Store, StoreError};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use thiserror::Error;
use tokio::sync::TryLockError;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{pk, ComponentId, DalContext, StandardModel, Timestamp, TransactionsError};

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type NodeResult<T> = Result<T, NodeError>;

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

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum NodeContent {
    V1(NodeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NodeContentV1 {
    pub timestamp: Timestamp,
    pub kind: NodeKind,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
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

impl Node {
    pub fn assemble(id: NodeId, inner: NodeContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            kind: inner.kind,
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height,
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub async fn new(
        ctx: &DalContext,
        kind: Option<NodeKind>,
        component_id: ComponentId,
    ) -> NodeResult<Self> {
        let content = NodeContentV1 {
            timestamp: Timestamp::now(),
            kind: match kind {
                Some(provided_kind) => provided_kind,
                None => NodeKind::Configuration,
            },
            x: "0".into(),
            y: "0".into(),
            width: None,
            height: None,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&NodeContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(&change_set, id, ContentAddress::Node(hash))?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
            workspace_snapshot.add_node(node_weight)?;

            // Component --> Node (this)
            workspace_snapshot.add_edge(
                component_id.into(),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                id,
            )?;
        }

        Ok(Self::assemble(id.into(), content))
    }

    // TODO(nick): restore the ability to create geometry.
    // async fn get_content(
    //     &mut self,
    //     ctx: &DalContext,
    //     node_id: NodeId,
    // ) -> WorkspaceSnapshotResult<(ContentHash, NodeContentV1)> {
    //     let id: Ulid = node_id.into();
    //     let node_index = self.working_copy()?.get_node_index_by_id(id)?;
    //     let node_weight = self.working_copy()?.get_node_weight(node_index)?;
    //     let hash = node_weight.content_hash();
    //
    //     let content: NodeContent = ctx
    //         .content_store()
    //         .lock()
    //         .await
    //         .get(&hash)
    //         .await?
    //         .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;
    //
    //     // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
    //     let inner = match content {
    //         NodeContentV1::V1(inner) => inner,
    //     };
    //
    //     Ok((hash, inner))
    // }
    //
    // pub async fn node_set_geometry(
    //     &mut self,
    //     ctx: &DalContext,
    //     change_set: &ChangeSetPointer,
    //     node_id: NodeId,
    //     x: impl AsRef<str>,
    //     y: impl AsRef<str>,
    //     width: Option<impl AsRef<str>>,
    //     height: Option<impl AsRef<str>>,
    // ) -> WorkspaceSnapshotResult<()> {
    //     let (_, inner) = self.node_get_content(ctx, node_id).await?;
    //
    //     let mut node = Node::assemble(node_id, &inner);
    //     node.x = x;
    //     node.y = y;
    //     node.width = width;
    //     node.height = height;
    //     let updated = NodeContentV1::from(node);
    //
    //     let hash = ctx
    //         .content_store()
    //         .lock()
    //         .await
    //         .add(&NodeContent::V1(updated.clone()))?;
    //
    //     self.working_copy()?
    //         .update_content(&change_set, node_id.into(), hash)?;
    //
    //     Ok(())
    // }
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
