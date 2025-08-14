use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    FuncId,
    SchemaVariantId,
    workspace_snapshot::EntityKind,
};

use crate::reference::ReferenceKind;

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
    trigger_entity = EntityKind::OutOfGraph,
    reference_kind = ReferenceKind::CachedSchemaVariant,
)]
pub struct CachedSchemaVariant {
    pub id: SchemaVariantId,
    pub variant_id: SchemaVariantId,
    pub display_name: String,
    pub category: String,
    pub color: String,
    pub is_locked: bool,
    pub description: Option<String>,
    pub link: Option<String>,
    pub asset_func_id: FuncId,
    pub variant_func_ids: Vec<FuncId>,
    pub is_default_variant: bool,
    // Note: domain_props excluded for Phase 1 as specified in the brief
}

impl CachedSchemaVariant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        variant_id: SchemaVariantId,
        display_name: String,
        category: String,
        color: String,
        is_locked: bool,
        description: Option<String>,
        link: Option<String>,
        asset_func_id: FuncId,
        variant_func_ids: Vec<FuncId>,
        is_default_variant: bool,
    ) -> Self {
        Self {
            id: variant_id, // Use variant_id as the MV object ID
            variant_id,
            display_name,
            category,
            color,
            is_locked,
            description,
            link,
            asset_func_id,
            variant_func_ids,
            is_default_variant,
        }
    }
}
