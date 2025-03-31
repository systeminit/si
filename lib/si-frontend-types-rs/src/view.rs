use serde::Deserialize;
use serde::Serialize;
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
    si_frontend_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::View,
    reference_kind = ReferenceKind::View,
)]
pub struct View {
    pub id: ViewId,
    pub name: String,
    pub is_default: bool,
    #[serde(flatten)]
    pub timestamp: Timestamp,
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
#[mv(
    trigger_entity = EntityKind::CategoryView,
    reference_kind = ReferenceKind::ViewList,
)]
pub struct ViewList {
    pub id: ChangeSetId,
    #[mv(reference_kind = ReferenceKind::View)]
    pub views: Vec<Reference<ViewId>>,
}
