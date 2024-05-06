use serde::{Deserialize, Serialize};
use si_data_pg::postgres_types::ToSql;
use strum::{AsRefStr, Display, EnumString};

#[remain::sorted]
#[derive(
    AsRefStr, Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Clone, ToSql,
)]
pub enum ChangeSetStatus {
    Abandoned,
    Applied,
    Closed,
    Failed,
    NeedsAbandonApproval,
    NeedsApproval,
    Open,
}
