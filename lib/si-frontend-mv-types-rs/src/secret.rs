use serde::{
    Deserialize,
    Serialize,
};
use si_events::SecretId;

use crate::schema_variant::prop_tree::PropWidgetKind;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SecretFormDataView {
    pub name: String,
    pub kind: String,
    pub widget_kind: PropWidgetKind,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SecretDefinition {
    pub label: String,
    pub form_data: Vec<SecretFormDataView>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    /// The [`id`](SecretId) of a [`Secret`].
    pub id: SecretId,
    /// The name of a [`Secret`] as provided by the user.
    pub name: String,
    /// The definition of a [`Secret`].
    pub label: String,
    /// The description of a [`Secret`] as provided by the user.
    pub description: Option<String>,
    /// If the secret can be used on this workspace
    pub is_usable: bool,
}
