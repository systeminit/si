use std::collections::HashMap;

use serde::Serialize;
use si_events::workspace_snapshot::EntityKind;
use si_id::{
    ComponentId,
    WorkspacePk,
};

use crate::{
    component::{
        ComponentInList,
        component_diff::ComponentDiff,
    },
    reference::ReferenceKind,
};

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
  trigger_entity = EntityKind::CategoryComponent,
  reference_kind = ReferenceKind::ErasedComponents,
  build_priority = "List",
)]
pub struct ErasedComponents {
    pub id: WorkspacePk,
    pub erased: HashMap<ComponentId, HeadComponent>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct HeadComponent {
    pub diff: ComponentDiff,
    pub component: ComponentInList,
}
