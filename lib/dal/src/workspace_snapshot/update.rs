use petgraph::prelude::*;

use super::edge_weight::EdgeWeight;

#[derive(Debug, Clone)]
pub enum Update {
    NewSubgraph {
        source: NodeIndex,
    },
    NewEdge {
        source: NodeIndex,
        destination: NodeIndex,
        weight: EdgeWeight,
    },
    ReplaceSubgraph {
        new: NodeIndex,
        old: NodeIndex,
    },
}
