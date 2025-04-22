use std::collections::{
    HashMap,
    HashSet,
};

use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::ulid::Ulid;

pub mod v1;

pub use v1::DeprecatedWorkspaceSnapshotGraphV1;

use super::LineageId;
use crate::workspace_snapshot::{
    edge_weight::deprecated::DeprecatedEdgeWeightLegacy,
    node_weight::deprecated::DeprecatedNodeWeightLegacy,
};

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct DeprecatedWorkspaceSnapshotGraphLegacy {
    pub graph: StableDiGraph<DeprecatedNodeWeightLegacy, DeprecatedEdgeWeightLegacy>,
    pub node_index_by_id: HashMap<Ulid, NodeIndex>,
    pub node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
    pub root_index: NodeIndex,
}
