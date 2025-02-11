use serde::{Deserialize, Serialize};
use si_events::{
    workspace_snapshot::{Checksum, ChecksumHasher, EntityKind},
    Timestamp,
};
use si_id::{ChangeSetId, ViewId};

use crate::{
    checksum::FrontendChecksum,
    object::FrontendObject,
    reference::{Refer, Reference, ReferenceId, ReferenceKind},
    MaterializedView,
};

#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
    si_frontend_types_macros::Refer,
)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub id: ViewId,
    pub name: String,
    pub is_default: bool,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

// TODO: we should be able to derive this impl in the future with a macro
impl MaterializedView for View {
    fn reference_dependencies() -> &'static [ReferenceKind] {
        &[]
    }

    fn trigger_entity() -> EntityKind {
        EntityKind::View
    }
}

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
)]
pub struct ViewList {
    pub id: ChangeSetId,
    pub views: Vec<Reference<ViewId>>,
}

// TODO: we should be able to derive this impl in the future with a macro
impl MaterializedView for ViewList {
    fn reference_dependencies() -> &'static [ReferenceKind] {
        &[ReferenceKind::View]
    }

    fn trigger_entity() -> EntityKind {
        EntityKind::CategoryView
    }
}
