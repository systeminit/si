use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::{
    CustomEdgeWeight, CustomNodeWeight, EdgeKind, SplitGraphEdgeWeight, SplitGraphNodeId,
    SplitGraphNodeWeight,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumDiscriminants)]
pub enum Update<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    NewEdge {
        source: SplitGraphNodeId,
        destination: SplitGraphNodeId,
        edge_weight: SplitGraphEdgeWeight<E, K>,
    },
    RemoveEdge {
        source: SplitGraphNodeId,
        destination: SplitGraphNodeId,
        edge_kind: K,
    },
    RemoveNode {
        id: SplitGraphNodeId,
    },
    ReplaceNode {
        node_weight: SplitGraphNodeWeight<N>,
    },
    NewNode {
        node_weight: SplitGraphNodeWeight<N>,
    },
}
