mod v1;

use super::NodeWeightDiscriminants;
use crate::workspace_snapshot::node_weight::traits::SiVersionedNodeWeight;
use crate::workspace_snapshot::node_weight::view_node_weight::v1::ViewNodeWeightV1;
use serde::{Deserialize, Serialize};
use si_events::ulid::Ulid;
use si_events::ContentHash;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight,
)]
pub enum ViewNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(ViewNodeWeightV1),
}

impl ViewNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self::V1(ViewNodeWeightV1::new(id, lineage_id, content_hash))
    }
}
