use petgraph::prelude::*;

use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants, edge_weight::EdgeWeightKindDiscriminants,
        graph::WorkspaceSnapshotGraphResult, node_weight::NodeWeight,
    },
    SchemaId, SchemaVariantError, SchemaVariantId, WorkspaceSnapshotGraphV3,
};

impl WorkspaceSnapshotGraphV3 {
    pub fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotGraphResult<SchemaId> {
        let schema_variant_node_index = self
            .node_index_by_id
            .get(&schema_variant_id.into())
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))
            .map_err(Box::new)?;
        let incoming_edges = self
            .graph()
            .edges_directed(*schema_variant_node_index, Incoming);

        let mut schema_id: Option<SchemaId> = None;
        for edgeref in incoming_edges {
            if EdgeWeightKindDiscriminants::from(edgeref.weight().kind())
                == EdgeWeightKindDiscriminants::Use
            {
                if let NodeWeight::Content(content) = self.get_node_weight(edgeref.source())? {
                    if content.content_address_discriminants()
                        == ContentAddressDiscriminants::Schema
                    {
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
        }
        Ok(schema_id
            .ok_or(SchemaVariantError::SchemaNotFound(schema_variant_id))
            .map_err(Box::new)?)
    }
}
