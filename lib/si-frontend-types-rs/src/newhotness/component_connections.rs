use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::EntityKind;
use si_frontend_types_macros::{
    FrontendChecksum,
    FrontendObject,
    MV,
    Refer,
};
use si_id::{
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    InputSocketId,
    OutputSocketId,
    PropId,
};

use crate::reference::{
    Reference,
    ReferenceKind,
};

#[derive(
    Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize, FrontendChecksum,
)]
#[remain::sorted]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Connection {
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
    #[serde(rename_all = "camelCase")]
    Socket {
        from_component_id: ComponentId,
        from_attribute_value_id: AttributeValueId,
        from_attribute_value_path: String,
        from_socket_id: OutputSocketId,
        from_socket_name: String,
        to_component_id: ComponentId,
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
    reference_kind = ReferenceKind::ComponentConnectionsBeta,
)]
pub struct ComponentConnectionsBeta {
    pub id: ComponentId,
    pub incoming: Vec<Connection>,
    pub outgoing: Vec<Connection>,
}

#[derive(
    Clone, Debug, Deserialize, Eq, PartialEq, Serialize, FrontendChecksum, FrontendObject, Refer, MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
  trigger_entity = EntityKind::CategoryComponent,
  reference_kind = ReferenceKind::ComponentConnectionsListBeta,
)]
pub struct ComponentConnectionsListBeta {
    pub id: ChangeSetId,
    #[mv(reference_kind = ReferenceKind::ComponentConnectionsBeta)]
    pub component_connections: Vec<Reference<ComponentId>>,
}
