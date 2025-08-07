use serde::{
    Deserialize,
    Serialize,
};
use si_events::ulid::Ulid;
use si_id::UserPk;

use super::{
    NodeWeight,
    traits::SiVersionedNodeWeight,
};
use crate::DalContext;

pub mod v1;

pub use v1::ReasonNodeWeightV1;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight, Hash,
)]
pub enum ReasonNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(ReasonNodeWeightV1),
}

impl ReasonNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, reason: Reason) -> Self {
        Self::V1(ReasonNodeWeightV1::new(id, lineage_id, reason))
    }

    pub fn reason(&self) -> Reason {
        self.inner().reason
    }
}

/// What reason is expressed by this Reason node?
/// NOTE: this is postcard encoded, add new reasons to the end
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Reason {
    DefaultSubscription,
    UserAdded(Option<UserPk>),
}

impl Reason {
    pub fn new_user_added(ctx: &DalContext) -> Self {
        Self::UserAdded(ctx.history_actor().user_pk())
    }
}

impl Reason {
    pub fn new_reason_node(reason: Self) -> NodeWeight {
        let id = Ulid::new();
        let lineage_id = Ulid::new();
        NodeWeight::new_reason(id, lineage_id, reason)
    }
}
