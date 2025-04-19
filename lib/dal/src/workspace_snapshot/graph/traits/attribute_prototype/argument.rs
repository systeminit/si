use si_id::{
    AttributePrototypeArgumentId, ComponentId, InputSocketId, OutputSocketId, PropId, SecretId,
    StaticArgumentValueId,
};

use crate::workspace_snapshot::{
    content_address::ContentAddress, graph::WorkspaceSnapshotGraphResult,
};

#[remain::sorted]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArgumentValue {
    AttributeValueSubscription {
        component_id: ComponentId,
        json_pointer: ContentAddress,
    },
    InputSocket(InputSocketId),
    OutputSocket(OutputSocketId),
    Prop(PropId),
    Secret(SecretId),
    StaticArgumentValue(StaticArgumentValueId),
}

pub trait AttributePrototypeArgumentExt {
    fn argument_value(
        &self,
        apa_id: AttributePrototypeArgumentId,
    ) -> WorkspaceSnapshotGraphResult<Option<ArgumentValue>>;
}
