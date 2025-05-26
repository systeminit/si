use attribute_tree::AttributeTree;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetId,
    ComponentId,
    SchemaId,
    SchemaVariantId,
    workspace_snapshot::EntityKind,
};

use crate::reference::{
    Reference,
    ReferenceKind,
    WeakReference,
    weak,
};

pub mod attribute_tree;

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
pub struct ComponentQualificationStats {
    pub total: u64,
    pub warned: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub running: u64,
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
pub struct ComponentDiff {
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
    pub schema_variant_name: String,
    pub schema_variant_description: Option<String>,
    pub schema_variant_doc_link: Option<String>,
    pub schema_category: String,
    pub has_resource: bool,
    pub qualification_totals: ComponentQualificationStats,
    pub input_count: usize,
    pub diff_count: usize,
    pub attribute_tree: AttributeTree,
    pub resource_diff: ComponentDiff,
    pub is_secret_defining: bool,
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
  trigger_entity = EntityKind::CategoryComponent,
  reference_kind = ReferenceKind::ComponentList,
)]
pub struct ComponentList {
    pub id: ChangeSetId,
    #[mv(reference_kind = ReferenceKind::Component)]
    pub components: Vec<Reference<ComponentId>>,
}
