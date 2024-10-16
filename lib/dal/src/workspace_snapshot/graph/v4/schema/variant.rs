use petgraph::prelude::*;

use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphV4;
use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants, edge_weight::EdgeWeightKindDiscriminants,
        graph::WorkspaceSnapshotGraphResult, node_weight::NodeWeight,
    },
    SchemaId, SchemaVariantError, SchemaVariantId,
};

impl WorkspaceSnapshotGraphV4 {
    pub fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotGraphResult<SchemaId> {
        let schema_variant_node_index = self
            .node_index_by_id
            .get(&schema_variant_id.into())
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))
            .map_err(Box::new)?;
        let incoming_edges = self.edges_directed_for_edge_weight_kind(
            *schema_variant_node_index,
            Incoming,
            EdgeWeightKindDiscriminants::Use,
        );

        let mut schema_id: Option<SchemaId> = None;
        for (_edge_weight, source_node_index, _destination_node_index) in incoming_edges {
            if let NodeWeight::Content(content) = self.get_node_weight(source_node_index)? {
                if content.content_address_discriminants() == ContentAddressDiscriminants::Schema {
                    schema_id = match schema_id {
                        None => Some(content.id().into()),
                        Some(_already_found_schema_id) => {
                            return Err(Box::new(SchemaVariantError::MoreThanOneSchemaFound(
                                schema_variant_id,
                            ))
                            .into());
                        }
                    };
                }
            }
        }
        Ok(schema_id
            .ok_or(SchemaVariantError::SchemaNotFound(schema_variant_id))
            .map_err(Box::new)?)
    }
}
