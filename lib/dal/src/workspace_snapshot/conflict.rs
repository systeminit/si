use serde::{Deserialize, Serialize};

use crate::{workspace_snapshot::NodeInformation, EdgeWeightKindDiscriminants};

/// Describe the type of conflict between the given locations in a
/// workspace graph.
#[remain::sorted]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Conflict {
    ChildOrder {
        onto: NodeInformation,
        to_rebase: NodeInformation,
    },
    ExclusiveEdgeMismatch {
        source: NodeInformation,
        destination: NodeInformation,
        edge_kind: EdgeWeightKindDiscriminants,
    },
    ModifyRemovedItem(NodeInformation),
    NodeContent {
        onto: NodeInformation,
        to_rebase: NodeInformation,
    },
    RemoveModifiedItem {
        container: NodeInformation,
        removed_item: NodeInformation,
    },
}
