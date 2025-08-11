use lazy_static::lazy_static;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    SchemaId,
    workspace_snapshot::EntityKind,
};
use si_id::ulid::{
    CoreUlid,
    Ulid,
};

use crate::reference::ReferenceKind;

lazy_static! {
    static ref CACHED_SCHEMAS_STATIC_ULID: Ulid =
        CoreUlid::from_parts(1754943219, 907162023253385544972939).into();
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
    trigger_entity = EntityKind::OutOfGraph,
    reference_kind = ReferenceKind::CachedSchemas,
)]
pub struct CachedSchemas {
    pub id: Ulid,
    pub schemas: Vec<CachedSchema>,
}

impl CachedSchemas {
    pub fn new(schemas: Vec<CachedSchema>) -> Self {
        Self {
            id: *CACHED_SCHEMAS_STATIC_ULID,
            schemas,
        }
    }
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
pub struct CachedSchema {
    pub id: SchemaId,
    pub name: String,
}
