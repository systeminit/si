use async_trait::async_trait;

use crate::{
    workspace_snapshot::{
        graph::SchemaVariantExt as SchemaVariantExtGraph, SchemaId, SchemaVariantId,
        WorkspaceSnapshot, WorkspaceSnapshotResult,
    },
    InputSocketId,
};

#[async_trait]
pub trait SchemaVariantExt {
    /// Return the [`SchemaId`] for the provided [`SchemaVariantId`]. Errors if either the
    /// [`Schema`][crate::Schema], or the [`SchemaVariant`][crate::SchemaVariant] are not found.
    async fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<SchemaId>;

    async fn schema_variant_add_edge_to_input_socket(
        &self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<()>;
}

#[async_trait]
impl SchemaVariantExt for WorkspaceSnapshot {
    async fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<SchemaId> {
        self.working_copy()
            .await
            .schema_id_for_schema_variant_id(schema_variant_id)
            .map_err(Into::into)
    }

    async fn schema_variant_add_edge_to_input_socket(
        &self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut()
            .await
            .schema_variant_add_edge_to_input_socket(schema_variant_id, input_socket_id)?;

        Ok(())
    }
}
