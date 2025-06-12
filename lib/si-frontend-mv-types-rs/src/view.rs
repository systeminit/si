use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    Timestamp,
    workspace_snapshot::EntityKind,
};
use si_frontend_mv_types_macros::{
    FrontendChecksum,
    FrontendObject,
    MV,
    Refer,
};
use si_id::{
    ComponentId,
    ViewId,
    WorkspacePk,
};

use crate::reference::{
    Reference,
    ReferenceKind,
    WeakReference,
    weak,
};

#[derive(
    Clone, Debug, Deserialize, Serialize, Eq, PartialEq, FrontendChecksum, FrontendObject, Refer, MV,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq, FrontendChecksum, FrontendObject, Refer, MV)]
#[mv(
    trigger_entity = EntityKind::CategoryView,
    reference_kind = ReferenceKind::ViewList,
)]
pub struct ViewList {
    pub id: WorkspacePk,
    pub views: Vec<Reference<ViewId>>,
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
#[mv(
    trigger_entity = EntityKind::View,
    reference_kind = ReferenceKind::ViewComponentList,
)]
pub struct ViewComponentList {
    pub id: ViewId,
    pub components: Vec<WeakReference<ComponentId, weak::markers::Component>>,
}
