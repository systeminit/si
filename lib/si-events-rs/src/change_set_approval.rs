use postgres_types::ToSql;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

use crate::create_xxhash_type;

create_xxhash_type!(ChangesChecksum);

#[remain::sorted]
#[derive(
    AsRefStr, Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Copy, Clone, ToSql,
)]
pub enum ChangeSetApprovalStatus {
    Approved,
}

#[remain::sorted]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ChangeSetApprovalKind {
    Func,
    SchemaVariant,
    View,
}
