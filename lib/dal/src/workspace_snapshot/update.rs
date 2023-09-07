use petgraph::prelude::*;

use super::edge_weight::EdgeWeight;
use serde::{Deserialize, Serialize};

#[remain::sorted]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Update {
    NewEdge {
        source: NodeIndex,
        destination: NodeIndex,
        edge_weight: EdgeWeight,
    },
    NewSubgraph {
        source: NodeIndex,
    },
    RemoveEdge(EdgeIndex),
    ReplaceSubgraph {
        // "onto"
        new: NodeIndex,
        // "to_rebase"
        old: NodeIndex,
    },
}
