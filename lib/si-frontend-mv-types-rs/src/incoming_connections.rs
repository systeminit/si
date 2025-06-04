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
    InputSocketId,
    OutputSocketId,
    PropId,
    WorkspacePk,
};

use crate::reference::{
    Reference,
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
    Prop {
        from_component_id: WeakReference<ComponentId, weak::markers::Component>,
        from_attribute_value_id: AttributeValueId,
        from_attribute_value_path: String,
        from_prop_id: PropId,
        from_prop_path: String,
        to_component_id: WeakReference<ComponentId, weak::markers::Component>,
        to_prop_id: PropId,
        to_prop_path: String,
        to_attribute_value_id: AttributeValueId,
        to_attribute_value_path: String,
    },
    #[serde(rename_all = "camelCase")]
    Socket {
        from_component_id: WeakReference<ComponentId, weak::markers::Component>,
        from_attribute_value_id: AttributeValueId,
        from_attribute_value_path: String,
        from_socket_id: OutputSocketId,
        from_socket_name: String,
        to_component_id: WeakReference<ComponentId, weak::markers::Component>,
        to_socket_id: InputSocketId,
        to_socket_name: String,
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
  trigger_entity = EntityKind::CategoryComponent,
  reference_kind = ReferenceKind::IncomingConnectionsList,
)]
pub struct IncomingConnectionsList {
    pub id: WorkspacePk,
    #[mv(reference_kind = ReferenceKind::IncomingConnections)]
    pub component_connections: Vec<Reference<ComponentId>>,
}
