use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use si_events::workspace_snapshot::EntityKind;
use si_id::{
    ComponentId,
    SchemaId,
    SchemaVariantId,
    WorkspacePk,
};
use strum::{
    AsRefStr,
    Display,
    EnumString,
};

use crate::reference::{
    ReferenceKind,
    WeakReference,
    weak,
};

pub mod attribute_tree;
pub mod component_diff;
pub mod erased_components;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ComponentQualificationStats {
    pub total: u64,
    pub warned: u64,
    pub succeeded: u64,
    pub failed: u64,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ComponentTextDiff {
    pub current: Option<String>,
    pub diff: Option<String>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::Component,
    reference_kind = ReferenceKind::Component,
)]
pub struct Component {
    pub id: ComponentId,
    pub name: String,
    pub color: Option<String>,
    pub schema_name: String,
    pub schema_id: SchemaId,
    pub schema_variant_id: WeakReference<SchemaVariantId, weak::markers::SchemaVariant>,
    pub schema_members: WeakReference<SchemaId, weak::markers::SchemaMembers>,
    pub schema_variant_name: String,
    pub schema_variant_description: Option<String>,
    pub schema_variant_doc_link: Option<String>,
    pub schema_category: String,
    pub has_resource: bool,
    pub qualification_totals: ComponentQualificationStats,
    pub input_count: usize,
    pub resource_diff: ComponentTextDiff,
    pub is_secret_defining: bool,
    pub to_delete: bool,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Deserialize,
    Serialize,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Eq,
    Copy,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
pub enum ComponentDiffStatus {
    Added,
    Modified,
    None,
    Removed,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::Component,
    reference_kind = ReferenceKind::ComponentInList,
)]
pub struct ComponentInList {
    pub id: ComponentId,
    pub name: String,
    pub color: Option<String>,
    pub schema_name: String,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_variant_name: String,
    pub schema_category: String,
    pub has_resource: bool,
    pub resource_id: Option<Value>,
    pub qualification_totals: ComponentQualificationStats,
    pub input_count: usize,
    pub diff_status: ComponentDiffStatus,
    pub to_delete: bool,
    // TODO (jkeiser) remove (this is always false)
    pub has_socket_connections: bool,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::Schema,
    reference_kind = ReferenceKind::SchemaMembers,
)]
pub struct SchemaMembers {
    pub id: SchemaId,
    pub default_variant_id: SchemaVariantId,
    pub editing_variant_id: Option<SchemaVariantId>,
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
  trigger_entity = EntityKind::CategoryComponent,
  reference_kind = ReferenceKind::ComponentList,
  build_priority = "List",
)]
pub struct ComponentList {
    pub id: WorkspacePk,
    pub components: Vec<WeakReference<ComponentId, weak::markers::ComponentInList>>,
}
