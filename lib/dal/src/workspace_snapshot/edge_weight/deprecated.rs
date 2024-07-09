use serde::{Deserialize, Serialize};

use crate::{workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock, EdgeWeightKind};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DeprecatedEdgeWeight {
    kind: EdgeWeightKind,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
}
