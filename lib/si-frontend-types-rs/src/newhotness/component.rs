use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ComponentId,
    SchemaId,
    SchemaVariantId,
    workspace_snapshot::{
        Checksum,
        ChecksumHasher,
        EntityKind,
    },
};
use si_id::ChangeSetId;

use crate::{
    MaterializedView,
    checksum::FrontendChecksum,
    object::FrontendObject,
    reference::{
        Refer,
        Reference,
        ReferenceId,
        ReferenceKind,
    },
};

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, Clone, si_frontend_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ComponentQualificationTotals {
    pub total: i64,
    pub warned: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub running: i64,
}

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, Clone, si_frontend_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ComponentView {
    pub id: ComponentId,
    pub name: String,
    pub schema_name: String,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_variant_name: String,
    pub schema_category: String,
    pub has_resource: bool,
    pub qualification_totals: ComponentQualificationTotals,
    pub input_count: usize,
    pub output_count: usize,
    pub diff_count: usize,
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
  trigger_entity = EntityKind::CategoryComponent,
  reference_kind = ReferenceKind::ComponentViewList,
)]
pub struct ComponentViewList {
    pub id: ChangeSetId,
    pub components: Vec<ComponentView>,
}
