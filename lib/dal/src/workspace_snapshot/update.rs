use petgraph::prelude::*;

use super::edge_weight::EdgeWeight;

#[remain::sorted]
#[derive(Debug, Clone)]
pub enum Update {
    NewEdge {
        source: NodeIndex,
        destination: NodeIndex,
        weight: EdgeWeight,
    },
    NewSubgraph {
        source: NodeIndex,
    },
    RemoveEdge(EdgeIndex),
    ReplaceSubgraph {
        new: NodeIndex,
        old: NodeIndex,
    },
}
