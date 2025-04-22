use dal_macros::SiVersionedNodeWeight;
use jwt_simple::prelude::{
    Deserialize,
    Serialize,
};
use si_events::ulid::Ulid;
use strum::Display;

use crate::{
    diagram::view::ViewId,
    workspace_snapshot::node_weight::traits::SiVersionedNodeWeight,
};

pub mod v1;
use v1::DiagramObjectNodeWeightV1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiVersionedNodeWeight)]
pub enum DiagramObjectNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(DiagramObjectNodeWeightV1),
}

impl DiagramObjectNodeWeight {
    pub fn object_kind(&self) -> DiagramObjectKind {
        self.inner().object_kind()
    }

    pub fn new(id: Ulid, lineage_id: Ulid, object_kind: DiagramObjectKind) -> Self {
        Self::V1(DiagramObjectNodeWeightV1::new(id, lineage_id, object_kind))
    }
}

/// Represents the type of object the diagram node will represent.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Display)]
pub enum DiagramObjectKind {
    View(ViewId),
}
