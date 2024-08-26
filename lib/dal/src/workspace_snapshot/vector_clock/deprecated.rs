use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{id, workspace_snapshot::lamport_clock::LamportClock};

id!(DeprecatedVectorClockId);

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct DeprecatedVectorClock {
    pub entries: HashMap<DeprecatedVectorClockId, LamportClock>,
}
