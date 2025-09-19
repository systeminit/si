use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    SchemaId,
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
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::OutOfGraph,
    reference_kind = ReferenceKind::CachedSchema,
)]
pub struct CachedSchema {
    pub id: SchemaId,
    pub name: String,
    pub default_variant_id: SchemaVariantId,
    pub variant_ids: Vec<SchemaVariantId>,
}

impl CachedSchema {
    pub fn new(
        id: SchemaId,
        name: String,
        default_variant_id: SchemaVariantId,
        variant_ids: Vec<SchemaVariantId>,
    ) -> Self {
        Self {
            id,
            name,
            default_variant_id,
            variant_ids,
        }
    }
}
