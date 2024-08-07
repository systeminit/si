use serde::{Deserialize, Serialize};

use crate::{workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock, EdgeWeightKind};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DeprecatedEdgeWeightLegacy {
    pub kind: EdgeWeightKind,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
}
