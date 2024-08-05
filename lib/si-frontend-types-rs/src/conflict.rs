use serde::{Deserialize, Serialize};
use si_events::AttributeValueId;
use strum::{AsRefStr, Display, EnumIter, EnumString};

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Debug,
    Deserialize,
    EnumString,
    Eq,
    Serialize,
    Display,
    EnumIter,
    PartialEq,
    Hash,
)]
#[serde(rename_all = "camelCase", tag = "bindingKind")]
pub enum ConflictWithHead {
    #[serde(rename_all = "camelCase")]
    ModifiedWhatHeadRemoved { modified_av_id: AttributeValueId },
    #[serde(rename_all = "camelCase")]
    RemovedWhatHeadModified { container_av_id: AttributeValueId },
    #[serde(rename_all = "camelCase")]
    Untreated { raw: String },
}
