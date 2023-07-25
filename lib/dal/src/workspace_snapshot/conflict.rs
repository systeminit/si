use petgraph::stable_graph::NodeIndex;

/// Describe the type of conflict between the given locations in a
/// workspace graph.
#[remain::sorted]
#[derive(Debug, Copy, Clone)]
pub enum Conflict {
    ChildMembership(ConflictLocation),
    ChildOrder(ConflictLocation),
    NodeContent(ConflictLocation),
}

/// The [`NodeIndex`] of the location in the graph where a conflict occurs.
#[derive(Debug, Copy, Clone)]
pub struct ConflictLocation {
    /// The location of the conflict in the "base" graph of the merge.
    pub base: NodeIndex,
    /// The location of the conflict in the graph that is attempting to be merged into "base".
    pub other: NodeIndex,
}
