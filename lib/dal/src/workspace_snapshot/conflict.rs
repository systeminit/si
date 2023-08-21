use petgraph::stable_graph::NodeIndex;

/// Describe the type of conflict between the given locations in a
/// workspace graph.
#[remain::sorted]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Conflict {
    ChildOrder {
        ours: NodeIndex,
        theirs: NodeIndex,
    },
    ModifyRemovedItem(NodeIndex),
    NodeContent {
        to_rebase: NodeIndex,
        onto: NodeIndex,
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
