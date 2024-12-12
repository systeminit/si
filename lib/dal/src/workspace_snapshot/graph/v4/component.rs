use anyhow::Result;
use petgraph::prelude::*;
use si_id::ViewId;

use crate::{
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants, graph::WorkspaceSnapshotGraphV4,
        node_weight::NodeWeightDiscriminants,
    },
    ComponentError, ComponentId, SchemaVariantId,
};

impl WorkspaceSnapshotGraphV4 {
    pub fn frame_contains_components(&self, component_id: ComponentId) -> Result<Vec<ComponentId>> {
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
            let node_weight = self.get_node_weight(destination_node_index)?;
            if NodeWeightDiscriminants::from(node_weight) == NodeWeightDiscriminants::Component {
                results.push(node_weight.id().into());
            }
        }

        Ok(results)
    }

    pub fn schema_variant_id_for_component_id(
        &self,
        component_id: ComponentId,
    ) -> Result<SchemaVariantId> {
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
            let node_weight = self.get_node_weight(destination_node_index)?;
            if NodeWeightDiscriminants::from(node_weight) == NodeWeightDiscriminants::SchemaVariant
            {
                return Ok(node_weight.id().into());
            }
        }

        Err(ComponentError::SchemaVariantNotFound(component_id).into())
    }

    pub fn component_contained_in_views(&self, component_id: ComponentId) -> Result<Vec<ViewId>> {
        // Component <--Represents-- Geometry <--Use-- View
        let component_node_index = *self
            .node_index_by_id
            .get(&component_id.into())
            .ok_or(ComponentError::NotFound(component_id))
            .map_err(Box::new)?;
        let mut results = Vec::new();
        for (_edge_weight, geometry_node_index, _component_node_index) in self
            .edges_directed_for_edge_weight_kind(
                component_node_index,
                Incoming,
                EdgeWeightKindDiscriminants::Represents,
            )
        {
            let view_node_index = self.get_edge_weight_kind_target_idx(
                geometry_node_index,
                Incoming,
                EdgeWeightKindDiscriminants::Use,
            )?;
            let view_id = self.get_node_weight(view_node_index)?.id();
            results.push(view_id.into());
        }

        Ok(results)
    }
}
