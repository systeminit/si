use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    ulid::Ulid,
};

use super::traits::SiVersionedNodeWeight;
use crate::{
    func::leaf::LeafKind,
    workspace_snapshot::content_address::ContentAddress,
};

pub mod v1;

pub use v1::LeafPrototypeNodeWeightV1;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight, Hash,
)]
pub enum LeafPrototypeNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(LeafPrototypeNodeWeightV1),
}

impl LeafPrototypeNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, kind: LeafKind, inputs: ContentHash) -> Self {
        Self::V1(LeafPrototypeNodeWeightV1::new(id, lineage_id, kind, inputs))
    }

    pub fn inputs(&self) -> ContentAddress {
        self.inner().content_address
    }

    pub fn kind(&self) -> LeafKind {
        self.inner().kind
    }
}
