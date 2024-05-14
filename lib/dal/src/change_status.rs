use serde::Deserialize;
use serde::Serialize;
use strum::{AsRefStr, Display, EnumString};

/// An enum representing the changez status of an entity in the [`ChangeSet`](crate::ChangeSet).
#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy, Display, EnumString, AsRefStr,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ChangeStatus {
    Added,
    Deleted,
    Modified,
    Unmodified,
}
