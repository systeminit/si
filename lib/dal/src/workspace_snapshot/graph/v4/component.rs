use petgraph::prelude::*;

use crate::{
    component::ComponentResult,
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants, graph::WorkspaceSnapshotGraphV4,
        node_weight::NodeWeightDiscriminants,
    },
    ComponentError, ComponentId, SchemaVariantId, WorkspaceSnapshotError,
};

impl WorkspaceSnapshotGraphV4 {
    pub fn frame_contains_components(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        let component_node_index = *self
            .node_index_by_id
            .get(&component_id.into())
            .ok_or(ComponentError::NotFound(component_id))?;
        let mut results = Vec::new();
        for (_edge_weight, _source_node_index, destination_node_index) in self
            .edges_directed_for_edge_weight_kind(
                component_node_index,
                Outgoing,
                EdgeWeightKindDiscriminants::FrameContains,
            )
        {
            let node_weight = self
                .get_node_weight(destination_node_index)
                .map_err(WorkspaceSnapshotError::from)?;
            if NodeWeightDiscriminants::from(node_weight) == NodeWeightDiscriminants::Component {
                results.push(node_weight.id().into());
            }
        }

        Ok(results)
    }

    pub fn schema_variant_id_for_component_id(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariantId> {
        let component_node_index = *self
            .node_index_by_id
            .get(&component_id.into())
            .ok_or(ComponentError::NotFound(component_id))?;
        for (_edge_weight, _source_node_index, destination_node_index) in self
            .edges_directed_for_edge_weight_kind(
                component_node_index,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
        {
            let node_weight = self
                .get_node_weight(destination_node_index)
                .map_err(WorkspaceSnapshotError::from)?;
            if NodeWeightDiscriminants::from(node_weight) == NodeWeightDiscriminants::SchemaVariant
            {
                return Ok(node_weight.id().into());
            }
        }

        Err(ComponentError::SchemaVariantNotFound(component_id))
    }
}
