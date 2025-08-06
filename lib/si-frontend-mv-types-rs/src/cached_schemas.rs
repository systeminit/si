use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    SchemaId,
    workspace_snapshot::EntityKind,
};
use si_id::ulid::Ulid;

use crate::reference::{
    ReferenceKind,
};

const CACHED_SCHEMAS_STATIC_ID_STR: &str = "01K1Y078734YR40692NNSQRKG4";

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
            id: Self::STATIC_ID,
            schemas,
        }
    }
    pub const STATIC_ID: Ulid = match Ulid::from_string(CACHED_SCHEMAS_STATIC_ID_STR) {
        Ok(id) => id,
        Err(_) => panic!("Invalid static id for cached schemas")
    };
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
