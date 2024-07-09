use std::collections::HashMap;

use crate::{pk, workspace_snapshot::lamport_clock::LamportClock};
use serde::{Deserialize, Serialize};

pk!(DeprecatedVectorClockId);

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct DeprecatedVectorClock {
    entries: HashMap<DeprecatedVectorClockId, LamportClock>,
}
