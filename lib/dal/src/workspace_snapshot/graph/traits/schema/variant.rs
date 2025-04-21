use crate::{
    InputSocketId, SchemaId, SchemaVariantId,
    workspace_snapshot::graph::WorkspaceSnapshotGraphResult,
};

pub trait SchemaVariantExt {
    fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotGraphResult<SchemaId>;

    fn schema_variant_add_edge_to_input_socket(
        &mut self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<()>;

    fn schema_variant_ids_for_schema_id_opt(
        &self,
        schema_id: SchemaId,
    ) -> WorkspaceSnapshotGraphResult<Option<Vec<SchemaVariantId>>>;
}
