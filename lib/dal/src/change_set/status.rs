use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::postgres_types::ToSql;
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

// NOTE(nick): if we can remove the "ToSql" trait, then we can fully move this to "si-events-rs"
// and delete the duplicate types.
#[remain::sorted]
#[derive(
    AsRefStr,
    Deserialize,
    Serialize,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Eq,
    Copy,
    Clone,
    ToSql,
    EnumIter,
)]
pub enum ChangeSetStatus {
    /// No longer usable
    Abandoned,
    /// Applied this changeset to its parent
    Applied,
    /// An apply has begun, but not yet finished
    ApplyStarted,
    /// Approved by relevant parties and ready to be applied
    Approved,
    /// Migration of Workspace Snapshot for this change set failed
    Failed,
    /// Planned to be abandoned but needs approval first
    /// todo(brit): Remove once rebac is done
    NeedsAbandonApproval,
    /// Planned to be applied but needs approval first
    NeedsApproval,
    /// Available for user's to modify
    Open,
    /// Request to apply was rejected
    Rejected,
}

impl ChangeSetStatus {
    pub fn is_active_or_applying(&self) -> bool {
        matches!(
            self,
            ChangeSetStatus::Open
                | ChangeSetStatus::NeedsApproval
                | ChangeSetStatus::NeedsAbandonApproval
                | ChangeSetStatus::Approved
                | ChangeSetStatus::Rejected
                | ChangeSetStatus::ApplyStarted
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            ChangeSetStatus::Open
                | ChangeSetStatus::NeedsApproval
                | ChangeSetStatus::NeedsAbandonApproval
                | ChangeSetStatus::Approved
                | ChangeSetStatus::Rejected
        )
    }
}

impl From<si_events::ChangeSetStatus> for ChangeSetStatus {
    fn from(value: si_events::ChangeSetStatus) -> Self {
        match value {
            si_events::ChangeSetStatus::Abandoned => Self::Abandoned,
            si_events::ChangeSetStatus::Applied => Self::Applied,
            si_events::ChangeSetStatus::Failed => Self::Failed,
            si_events::ChangeSetStatus::NeedsAbandonApproval => {
                ChangeSetStatus::NeedsAbandonApproval
            }
            si_events::ChangeSetStatus::NeedsApproval => Self::NeedsApproval,
            si_events::ChangeSetStatus::Open => Self::Open,
            si_events::ChangeSetStatus::Approved => Self::Approved,
            si_events::ChangeSetStatus::Rejected => Self::Rejected,
            si_events::ChangeSetStatus::ApplyStarted => Self::ApplyStarted,
        }
    }
}

impl From<ChangeSetStatus> for si_events::ChangeSetStatus {
    fn from(value: ChangeSetStatus) -> Self {
        match value {
            ChangeSetStatus::Abandoned => Self::Abandoned,
            ChangeSetStatus::Applied => Self::Applied,
            ChangeSetStatus::ApplyStarted => Self::ApplyStarted,
            ChangeSetStatus::Failed => Self::Failed,
            ChangeSetStatus::NeedsAbandonApproval => Self::NeedsAbandonApproval,
            ChangeSetStatus::NeedsApproval => Self::NeedsApproval,
            ChangeSetStatus::Open => Self::Open,
            ChangeSetStatus::Approved => Self::Approved,
            ChangeSetStatus::Rejected => Self::Rejected,
        }
    }
}
