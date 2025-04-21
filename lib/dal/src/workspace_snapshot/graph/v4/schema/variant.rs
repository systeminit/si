use petgraph::prelude::*;

use crate::{
    EdgeWeight, EdgeWeightKind, InputSocketId, SchemaId, SchemaVariantError, SchemaVariantId,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::EdgeWeightKindDiscriminants,
        graph::{SchemaVariantExt, WorkspaceSnapshotGraphResult, WorkspaceSnapshotGraphV4},
        node_weight::NodeWeight,
    },
};

impl SchemaVariantExt for WorkspaceSnapshotGraphV4 {
    fn schema_id_for_schema_variant_id(
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

    fn schema_variant_add_edge_to_input_socket(
        &mut self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<()> {
        self.add_edge_between_ids(
            schema_variant_id.into(),
            EdgeWeight::new(EdgeWeightKind::Socket),
            input_socket_id.into(),
        )?;

        Ok(())
    }

    fn schema_variant_ids_for_schema_id_opt(
        &self,
        schema_id: SchemaId,
    ) -> WorkspaceSnapshotGraphResult<Option<Vec<SchemaVariantId>>> {
        let schema_node_index = match self.node_index_by_id.get(&schema_id.into()) {
            Some(schema_idx) => *schema_idx,
            None => return Ok(None),
        };

        let mut result = Vec::new();
        for (_edge_weight, _schema_node_idx, schema_variant_idx) in self
            .edges_directed_for_edge_weight_kind(
                schema_node_index,
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
        {
            let variant_node_weight = self.get_node_weight(schema_variant_idx)?;
            result.push(variant_node_weight.id().into());
        }

        Ok(Some(result))
    }
}
