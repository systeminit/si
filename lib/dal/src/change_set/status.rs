use serde::{Deserialize, Serialize};
use si_data_pg::postgres_types::ToSql;
use strum::{Display, EnumString};

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Clone, ToSql)]
pub enum ChangeSetStatus {
    Abandoned,
    Applied,
    Closed,
    Failed,
    NeedsAbandonApproval,
    NeedsApproval,
    Open,
}
