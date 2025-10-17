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

use crate::{
    management::MgmtPrototypeView,
    reference::ReferenceKind,
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
