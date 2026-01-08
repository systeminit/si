use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::EntityKind;
use si_id::{
    ComponentId,
    WorkspacePk,
};

use crate::reference::ReferenceKind;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::MV,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::CategoryDependentValueRoots,
    reference_kind = ReferenceKind::DependentValueComponentList,
    build_priority = "List",
)]
pub struct DependentValueComponentList {
    pub id: WorkspacePk,
    pub component_ids: Vec<ComponentId>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::MV,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::CategoryDependentValueRoots,
    reference_kind = ReferenceKind::DependentValues,
    build_priority = "List",
)]
pub struct DependentValues {
    pub id: WorkspacePk,
    /// Mapping from component ID to the list of "dirty" attribute paths in that component.
    pub component_attributes: HashMap<ComponentId, Vec<String>>,
}
