use crate::{
    workspace_snapshot::{graph::WorkspaceSnapshotGraphResult, node_weight::InputSocketNodeWeight},
    InputSocketId,
};

pub trait InputSocketExt {
    fn get_input_socket(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight>;
}
