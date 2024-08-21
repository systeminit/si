use serde::Deserialize;
use serde::Serialize;
use strum::{AsRefStr, Display, EnumString};

use si_frontend_types::ChangeStatus as FeChangeStatus;

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

impl From<ChangeStatus> for FeChangeStatus {
    fn from(value: ChangeStatus) -> Self {
        match value {
            ChangeStatus::Added => FeChangeStatus::Added,
            ChangeStatus::Deleted => FeChangeStatus::Deleted,
            ChangeStatus::Modified => FeChangeStatus::Modified,
            ChangeStatus::Unmodified => FeChangeStatus::Unmodified,
        }
    }
}
