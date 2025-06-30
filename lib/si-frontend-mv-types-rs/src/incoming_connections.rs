use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::EntityKind;
use si_frontend_mv_types_macros::{
    FrontendChecksum,
    FrontendObject,
    MV,
    Refer,
};
use si_id::{
    AttributeValueId,
    ComponentId,
    PropId,
    WorkspacePk,
};

use crate::reference::{
    ReferenceKind,
    WeakReference,
    weak,
};

#[derive(
    Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize, FrontendChecksum,
)]
#[remain::sorted]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Connection {
    #[serde(rename_all = "camelCase")]
    Management {
        from_component_id: ComponentId,
        to_component_id: ComponentId,
    },
    #[serde(rename_all = "camelCase")]
    Prop {
        from_component_id: ComponentId,
        from_attribute_value_id: AttributeValueId,
        from_attribute_value_path: String,
        from_prop_id: PropId,
        from_prop_path: String,
        to_component_id: ComponentId,
        to_prop_id: PropId,
        to_prop_path: String,
        to_attribute_value_id: AttributeValueId,
        to_attribute_value_path: String,
    },
}

#[derive(
    Clone, Debug, Deserialize, Eq, PartialEq, Serialize, FrontendChecksum, FrontendObject, Refer, MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::Component,
    reference_kind = ReferenceKind::IncomingConnections,
)]
pub struct IncomingConnections {
    pub id: ComponentId,
    pub connections: Vec<Connection>,
}

#[derive(
    Clone, Debug, Deserialize, Eq, PartialEq, Serialize, FrontendChecksum, FrontendObject, Refer, MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::Component,
    reference_kind = ReferenceKind::ManagementConnections,
)]
pub struct ManagementConnections {
    pub id: ComponentId,
    pub connections: Vec<Connection>,
}

#[derive(
    Clone, Debug, Deserialize, Eq, PartialEq, Serialize, FrontendChecksum, FrontendObject, Refer, MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
  trigger_entity = EntityKind::CategoryComponent,
  reference_kind = ReferenceKind::IncomingConnectionsList,
  build_priority = "List",
)]
pub struct IncomingConnectionsList {
    pub id: WorkspacePk,
    pub component_connections: Vec<WeakReference<ComponentId, weak::markers::IncomingConnections>>,
}
