use serde::Deserialize;
use serde::Serialize;
use si_events::{
    workspace_snapshot::{Checksum, ChecksumHasher, EntityKind},
    Timestamp,
};
use si_id::{ChangeSetId, ViewId};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::{
    checksum::FrontendChecksum,
    object::FrontendObject,
    reference::{Refer, Reference, ReferenceId, ReferenceKind},
    MaterializedView, SchemaVariant, UninstalledVariant,
};

#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
    si_frontend_types_macros::Refer,
    si_frontend_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::View,
    reference_kind = ReferenceKind::View,
)]
pub struct View {
    pub id: ViewId,
    pub name: String,
    pub is_default: bool,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
    si_frontend_types_macros::Refer,
    si_frontend_types_macros::MV,
)]
#[mv(
    trigger_entity = EntityKind::CategoryView,
    reference_kind = ReferenceKind::ViewList,
)]
pub struct ViewList {
    pub id: ChangeSetId,
    #[mv(reference_kind = ReferenceKind::View)]
    pub views: Vec<Reference<ViewId>>,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Display,
    Serialize,
    Eq,
    PartialEq,
    si_frontend_types_macros::FrontendChecksum,
)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Variant {
    SchemaVariant(SchemaVariant),
    UninstalledVariant(UninstalledVariant),
}

#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
    si_frontend_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub enum VariantType {
    #[serde(alias = "Installed")]
    #[strum(serialize = "Installed")]
    Installed,
    #[serde(alias = "Uninstalled")]
    #[strum(serialize = "Uninstalled")]
    Uninstalled,
}

#[derive(
    Clone, Debug, Deserialize, Serialize, Eq, PartialEq, si_frontend_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct DisambiguateVariant {
    #[serde(rename = "type")]
    pub variant_type: VariantType,
    pub id: String,
    pub variant: Variant,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantsByCategory {
    pub display_name: String,
    pub schema_variants: Vec<DisambiguateVariant>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
    si_frontend_types_macros::Refer,
    si_frontend_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::CategorySchema,
    reference_kind = ReferenceKind::SchemaVariantCategories,
)]
pub struct SchemaVariantCategories {
    pub id: ChangeSetId,
    pub categories: Vec<SchemaVariantsByCategory>,
}
