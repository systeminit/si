use serde::{Deserialize, Serialize};
use si_data_pg::postgres_types::ToSql;
use strum::{AsRefStr, Display, EnumString};

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
