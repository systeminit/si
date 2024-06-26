use si_events::ulid::Ulid;

use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use super::edge_weight::{EdgeWeight, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::NodeInformation;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumDiscriminants)]
pub enum Update {
    NewEdge {
        source: NodeInformation,
        // Check if already exists in "onto" (source). Grab node weight from "to_rebase"
        // (destination) and see if there is an equivalent node (id and lineage) in "onto".
        // If not, use "import_subgraph".
        destination: NodeInformation,
        edge_weight: EdgeWeight,
    },
    RemoveEdge {
        source: NodeInformation,
        destination: NodeInformation,
        edge_kind: EdgeWeightKindDiscriminants,
    },
    // This is not correctly named. We really only want to replace the single node, as we also
    // generate Update entries to handle processing the rest of the subgraph.
    ReplaceSubgraph {
        onto: NodeInformation,
        // Check if already exists in "onto". Grab node weight from "to_rebase" and see if there is
        // an equivalent node (id and lineage) in "onto". If not, use "import_subgraph".
        to_rebase: NodeInformation,
    },
    MergeCategoryNodes {
        to_rebase_category_id: Ulid,
        onto_category_id: Ulid,
    },
}
