use crate::{
    workspace_snapshot::{
        graph::{traits::socket::input::InputSocketExt, WorkspaceSnapshotGraphResult},
        node_weight::{InputSocketNodeWeight, NodeWeight, NodeWeightError},
    },
    NodeWeightDiscriminants, WorkspaceSnapshotGraphV3,
};

impl InputSocketExt for WorkspaceSnapshotGraphV3 {
    fn get_input_socket(
        &self,
        input_socket_id: crate::InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight> {
        let node_weight = self.get_node_weight_by_id(input_socket_id)?;

        match node_weight {
            NodeWeight::InputSocket(input_socket_node_weight) => Ok(input_socket_node_weight),
            unexpected => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                unexpected.into(),
                NodeWeightDiscriminants::InputSocket,
            )
            .into()),
        }
    }
}
