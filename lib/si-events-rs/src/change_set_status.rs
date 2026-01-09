use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumString,
};

#[remain::sorted]
#[derive(
    AsRefStr, Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Copy, Clone,
)]
pub enum ChangeSetStatus {
    Abandoned,
    Applied,
    ApplyStarted,
    Approved,
    Failed,
    NeedsAbandonApproval,
    NeedsApproval,
    Open,
    Rejected,
}
