use si_events::ContentHash;

use crate::{
    workspace_snapshot::{
        graph::{LineageId, WorkspaceSnapshotGraphResult},
        node_weight::InputSocketNodeWeight,
    },
    InputSocketId, SchemaVariantId, SocketArity,
};

pub trait InputSocketExt {
    fn new_input_socket(
        &mut self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
        lineage_id: LineageId,
        arity: SocketArity,
        content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight>;

    fn get_input_socket(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight>;

    fn list_input_sockets_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotGraphResult<Vec<InputSocketNodeWeight>>;
}
