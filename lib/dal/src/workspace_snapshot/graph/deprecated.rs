use std::collections::{HashMap, HashSet};

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::ulid::Ulid;

use crate::workspace_snapshot::{
    edge_weight::deprecated::DeprecatedEdgeWeight, node_weight::deprecated::DeprecatedNodeWeight,
};

use super::LineageId;

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct DeprecatedWorkspaceSnapshotGraph {
    pub graph: StableDiGraph<DeprecatedNodeWeight, DeprecatedEdgeWeight>,
    pub node_index_by_id: HashMap<Ulid, NodeIndex>,
    pub node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
    pub root_index: NodeIndex,
}
