use serde::{Deserialize, Serialize};
use si_data_pg::postgres_types::ToSql;
use strum::{AsRefStr, Display, EnumString};

// NOTE(nick): if we can remove the "ToSql" trait, then we can fully move this to "si-events-rs"
// and delete the duplicate types.
#[remain::sorted]
#[derive(
    AsRefStr, Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Copy, Clone, ToSql,
)]
pub enum ChangeSetStatus {
    /// No longer usable
    Abandoned,
    /// Applied this changeset to its parent
    Applied,
    /// TODO appears to be unused
    Closed,
    /// TODO appears to be unused
    Failed,
    /// Planned to be abandoned but needs approval first
    NeedsAbandonApproval,
    /// Planned to be applied but needs approval first
    NeedsApproval,
    /// Normal state: potentially usable
    Open,
}

impl From<si_events::ChangeSetStatus> for ChangeSetStatus {
    fn from(value: si_events::ChangeSetStatus) -> Self {
        match value {
            si_events::ChangeSetStatus::Abandoned => Self::Abandoned,
            si_events::ChangeSetStatus::Applied => Self::Applied,
            si_events::ChangeSetStatus::Closed => Self::Closed,
            si_events::ChangeSetStatus::Failed => Self::Failed,
            si_events::ChangeSetStatus::NeedsAbandonApproval => {
                ChangeSetStatus::NeedsAbandonApproval
            }
            si_events::ChangeSetStatus::NeedsApproval => Self::NeedsApproval,
            si_events::ChangeSetStatus::Open => Self::Open,
        }
    }
}

impl From<ChangeSetStatus> for si_events::ChangeSetStatus {
    fn from(value: ChangeSetStatus) -> Self {
        match value {
            ChangeSetStatus::Abandoned => Self::Abandoned,
            ChangeSetStatus::Applied => Self::Applied,
            ChangeSetStatus::Closed => Self::Closed,
            ChangeSetStatus::Failed => Self::Failed,
            ChangeSetStatus::NeedsAbandonApproval => Self::NeedsAbandonApproval,
            ChangeSetStatus::NeedsApproval => Self::NeedsApproval,
            ChangeSetStatus::Open => Self::Open,
        }
    }
}
