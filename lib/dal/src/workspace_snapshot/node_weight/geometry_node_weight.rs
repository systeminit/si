mod v1;

use super::NodeWeightDiscriminants;
use crate::workspace_snapshot::node_weight::geometry_node_weight::v1::GeometryNodeWeightV1;
use crate::workspace_snapshot::node_weight::traits::SiVersionedNodeWeight;
use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use si_events::ulid::Ulid;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight,
)]
pub enum GeometryNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(GeometryNodeWeightV1),
}

impl GeometryNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self::V1(GeometryNodeWeightV1::new(id, lineage_id, content_hash))
    }
}
