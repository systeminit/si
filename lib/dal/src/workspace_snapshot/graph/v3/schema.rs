use petgraph::prelude::*;

use crate::{
    workspace_snapshot::graph::{WorkspaceSnapshotGraphError, WorkspaceSnapshotGraphResult},
    EdgeWeightKindDiscriminants, SchemaId, SchemaVariantId, WorkspaceSnapshotGraphV3,
};

pub mod variant;

impl WorkspaceSnapshotGraphV3 {
    pub fn schema_variant_ids_for_schema_id(
        &self,
        schema_id: SchemaId,
    ) -> WorkspaceSnapshotGraphResult<Vec<SchemaVariantId>> {
        self.schema_variant_ids_for_schema_id_opt(schema_id)?
            .ok_or_else(|| WorkspaceSnapshotGraphError::NodeWithIdNotFound(schema_id.into()))
    }

    pub fn schema_variant_ids_for_schema_id_opt(
        &self,
        schema_id: SchemaId,
    ) -> WorkspaceSnapshotGraphResult<Option<Vec<SchemaVariantId>>> {
        let mut schema_variant_ids = Vec::new();
        if let Some(schema_node_idx) = self.get_node_index_by_id_opt(schema_id) {
            for (_, _source_idx, destination_idx) in self.edges_directed_for_edge_weight_kind(
                schema_node_idx,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            ) {
                let dest_node_weight = self.get_node_weight(destination_idx)?;
                if dest_node_weight.get_schema_variant_node_weight().is_ok() {
                    schema_variant_ids.push(dest_node_weight.id().into());
                }
            }
        } else {
            return Ok(None);
        }

        Ok(Some(schema_variant_ids))
    }
}
