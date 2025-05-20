use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetId,
    SecretId,
    workspace_snapshot::EntityKind,
};
use si_id::PropId;

use crate::{
    reference::{
        ReferenceKind,
        WeakReference,
        weak,
    },
    schema_variant::prop_tree::PropWidgetKind,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, si_frontend_mv_types_macros::FrontendChecksum)]
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
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all(serialize = "camelCase"))]
#[mv(
  trigger_entity = EntityKind::Prop,
  reference_kind = ReferenceKind::SecretDefinition,
)]
pub struct SecretDefinition {
    pub id: PropId,
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
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::Secret,
    reference_kind = ReferenceKind::Secret,
)]
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
    /// The prop Id that contains the secret definition
    pub definition_id: WeakReference<PropId, weak::markers::SecretDefinition>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
  trigger_entity = EntityKind::CategorySchema,
  reference_kind = ReferenceKind::SecretDefinitionList,
)]
pub struct SecretDefinitionList {
    pub id: ChangeSetId,
    pub secret_definitions: Vec<WeakReference<PropId, weak::markers::SecretDefinition>>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
  trigger_entity = EntityKind::CategorySecret,
  reference_kind = ReferenceKind::SecretList,
)]
pub struct SecretList {
    pub id: ChangeSetId,
    #[mv(reference_kind = ReferenceKind::Secret)]
    pub secrets: Vec<WeakReference<SecretId, weak::markers::Secret>>,
}
