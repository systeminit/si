use petgraph::prelude::*;

use super::edge_weight::{EdgeWeight, EdgeWeightKindDiscriminants};
use serde::{Deserialize, Serialize};

#[remain::sorted]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Update {
    NewEdge {
        source: NodeIndex,
        // Check if already exists in "onto" (source). Grab node weight from "to_rebase"
        // (destination) and see if there is an equivalent node (id and lineage) in "onto".
        // If not, use "import_subgraph".
        destination: NodeIndex,
        edge_weight: EdgeWeight,
    },
    RemoveEdge {
        source: NodeIndex,
        destination: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    },
    ReplaceSubgraph {
        onto: NodeIndex,
        // Check if already exists in "onto". Grab node weight from "to_rebase" and see if there is
        // an equivalent node (id and lineage) in "onto". If not, use "import_subgraph".
        to_rebase: NodeIndex,
    },
}
