use std::collections::HashMap;

use prop_tree::PropTree;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    SchemaId,
    SchemaVariantId,
    Timestamp,
    workspace_snapshot::EntityKind,
};
use si_id::WorkspacePk;
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

use crate::{
    management::MgmtPrototypeView,
    reference::{
        ReferenceKind,
        WeakReference,
        weak,
    },
};

pub mod prop_tree;

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    Serialize,
    PartialEq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::SchemaVariant,
    reference_kind = ReferenceKind::SchemaVariant,
)]
pub struct SchemaVariant {
    pub id: SchemaVariantId,
    pub schema_id: SchemaId,
    pub schema_name: String,
    pub schema_variant_id: SchemaVariantId,
    pub version: String,
    pub display_name: String,
    pub category: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub color: String,
    pub is_locked: bool, // if unlocked, show in both places
    #[serde(flatten)]
    pub timestamp: Timestamp,
    pub can_create_new_components: bool, // if yes, show in modeling screen, if not, only show in customize
    pub is_secret_defining: bool,
    pub can_contribute: bool,
    pub mgmt_functions: Vec<MgmtPrototypeView>,
    pub prop_tree: PropTree,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    Serialize,
    PartialEq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct InstalledVariant {
    pub id: SchemaVariantId,
    pub schema_variant: WeakReference<SchemaVariantId, weak::markers::SchemaVariant>,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    Serialize,
    PartialEq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct UninstalledVariant {
    pub schema_id: SchemaId,
    pub schema_name: String,
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub link: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub is_locked: bool,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Display,
    Serialize,
    Eq,
    PartialEq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(untagged, rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum Variant {
    InstalledSchemaVariant(InstalledVariant),
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
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub enum VariantType {
    Installed,
    Uninstalled,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct DisambiguateVariant {
    #[serde(rename = "type")]
    pub variant_type: VariantType,
    pub id: String,
}

#[derive(
    Debug, Clone, Serialize, PartialEq, Eq, si_frontend_mv_types_macros::DefinitionChecksum,
)]
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
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::CategorySchema,
    reference_kind = ReferenceKind::SchemaVariantCategories,
    build_priority = "List",
)]
pub struct SchemaVariantCategories {
    pub id: WorkspacePk,
    pub categories: Vec<SchemaVariantsByCategory>,
    pub uninstalled: HashMap<SchemaId, UninstalledVariant>,
}
