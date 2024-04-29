use petgraph::stable_graph::NodeIndex;
use serde::Deserialize;
use serde::Serialize;

use crate::EdgeWeightKindDiscriminants;

/// Describe the type of conflict between the given locations in a
/// workspace graph.
#[remain::sorted]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Conflict {
    ChildOrder {
        onto: NodeIndex,
        to_rebase: NodeIndex,
    },
    ExclusiveEdgeMismatch {
        source: NodeIndex,
        destination: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    },
    ModifyRemovedItem(NodeIndex),
    NodeContent {
        onto: NodeIndex,
        to_rebase: NodeIndex,
    },
    RemoveModifiedItem {
        container: NodeIndex,
        removed_item: NodeIndex,
    },
}

/// The [`NodeIndex`] of the location in the graph where a conflict occurs.
#[derive(Debug, Copy, Clone)]
pub struct ConflictLocation {
    /// The location of the conflict in the "base" graph of the merge.
    pub onto: NodeIndex,
    /// The location of the conflict in the graph that is attempting to be merged into "base".
    pub to_rebase: NodeIndex,
}
