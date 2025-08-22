use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    FuncId,
    SchemaId,
    SchemaVariantId,
    workspace_snapshot::EntityKind,
};

use crate::{
    prop_schema::PropSchemaV1,
    reference::ReferenceKind,
};

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
    reference_kind = ReferenceKind::CachedDefaultVariant,
)]
pub struct CachedDefaultVariant {
    pub id: SchemaId, // Use schema_id as the MV object ID for direct lookup
    pub variant_id: SchemaVariantId,
    pub display_name: String,
    pub category: String,
    pub color: String,
    pub is_locked: bool,
    pub description: Option<String>,
    pub link: Option<String>,
    pub asset_func_id: FuncId,
    pub variant_func_ids: Vec<FuncId>,
    pub domain_props: Option<PropSchemaV1>,
    // is_default_variant always true for this MV, so not stored
}

impl CachedDefaultVariant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_id: SchemaId,
        variant_id: SchemaVariantId,
        display_name: String,
        category: String,
        color: String,
        is_locked: bool,
        description: Option<String>,
        link: Option<String>,
        asset_func_id: FuncId,
        variant_func_ids: Vec<FuncId>,
        domain_props: Option<PropSchemaV1>,
    ) -> Self {
        Self {
            id: schema_id, // Use schema_id as the MV object ID
            variant_id,
            display_name,
            category,
            color,
            is_locked,
            description,
            link,
            asset_func_id,
            variant_func_ids,
            domain_props,
        }
    }
}
