use postgres_types::ToSql;
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
    AsRefStr, Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Copy, Clone, ToSql,
)]
pub enum ChangeSetApprovalStatus {
    Approved,
    Rejected,
}
