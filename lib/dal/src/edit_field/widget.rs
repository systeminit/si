use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

#[derive(
    AsRefStr, Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Display, EnumString, Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WidgetKind {
    Array,
    Checkbox,
    Header,
    Map,
    SecretSelect,
    Text,
}
