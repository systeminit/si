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
    TextArea,
    /// Provides a select box for corresponding "primitive" (e.g. string, number, boolean)
    /// [`PropKinds`](crate::PropKind).
    Select,
    /// Provides a text input with auto-completion for corresponding "primitive" (e.g. string, number, boolean)
    /// [`PropKinds`](crate::PropKind).
    ComboBox,
}
