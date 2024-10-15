use async_trait::async_trait;

use crate::{
    layer_db_types::{InputSocketContent, InputSocketContentV2},
    socket::input::InputSocketResult,
    workspace_snapshot::{
        graph::InputSocketExt as InputSocketExtGraph,
        node_weight::{traits::SiVersionedNodeWeight, InputSocketNodeWeight},
        WorkspaceSnapshotResult,
    },
    DalContext, InputSocket, InputSocketId, WorkspaceSnapshot, WorkspaceSnapshotError,
};

#[async_trait]
pub trait InputSocketExt {
    /// Retrieve the [`InputSocket`] with the specified [`InputSocketId`].
    async fn get_input_socket(
        &self,
        ctx: &DalContext,
        id: InputSocketId,
    ) -> WorkspaceSnapshotResult<InputSocket>;
}

#[async_trait]
impl InputSocketExt for WorkspaceSnapshot {
    async fn get_input_socket(
        &self,
        ctx: &DalContext,
        id: InputSocketId,
    ) -> WorkspaceSnapshotResult<InputSocket> {
        let input_socket_node_weight = self.working_copy().await.get_input_socket(id)?;

        let content: InputSocketContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&input_socket_node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                input_socket_node_weight.id(),
            ))?;

        from_node_weight_and_content(&input_socket_node_weight, content)
            .map_err(Box::new)
            .map_err(Into::into)
    }
}

fn from_node_weight_and_content(
    node_weight: &InputSocketNodeWeight,
    content: InputSocketContent,
) -> InputSocketResult<InputSocket> {
    let input_socket = match content {
        InputSocketContent::V1(v1_inner) => {
            let v2_inner = InputSocketContentV2 {
                timestamp: v1_inner.timestamp,
                name: v1_inner.name.clone(),
                inbound_type_definition: v1_inner.inbound_type_definition.clone(),
                outbound_type_definition: v1_inner.outbound_type_definition.clone(),
                kind: v1_inner.kind,
                required: v1_inner.required,
                ui_hidden: v1_inner.ui_hidden,
                connection_annotations: v1_inner.connection_annotations.clone(),
            };

            InputSocket::assemble(node_weight.id().into(), v1_inner.arity, v2_inner)
        }
        InputSocketContent::V2(inner) => InputSocket::assemble(
            node_weight.id().into(),
            node_weight.inner().arity(),
            inner.clone(),
        ),
    };

    Ok(input_socket)
}
