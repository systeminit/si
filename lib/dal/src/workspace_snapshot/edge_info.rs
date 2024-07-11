use petgraph::prelude::{EdgeIndex, NodeIndex};

use crate::{EdgeWeightKindDiscriminants, WorkspaceSnapshotGraphV1};

#[derive(Debug, Copy, Clone)]
pub struct EdgeInfo {
    pub source_node_index: NodeIndex,
    pub target_node_index: NodeIndex,
    pub edge_kind: EdgeWeightKindDiscriminants,
    pub edge_index: EdgeIndex,
}

impl EdgeInfo {
    pub fn simple_debug_string(&self, graph: &WorkspaceSnapshotGraphV1) -> String {
        let source = graph.graph().node_weight(self.source_node_index);
        let target = graph.graph().node_weight(self.target_node_index);
        let edge_weight = graph.graph().edge_weight(self.edge_index);

        format!(
            "{:?} -- {:?} --> {:?}",
            source.map(|s| s.id()),
            edge_weight.map(|e| e.kind()),
            target.map(|t| t.id())
        )
    }
}
