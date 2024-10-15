use crate::{workspace_snapshot::graph::WorkspaceSnapshotGraphResult, SchemaId, SchemaVariantId};

pub trait SchemaVariantExt {
    fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotGraphResult<SchemaId>;
}
