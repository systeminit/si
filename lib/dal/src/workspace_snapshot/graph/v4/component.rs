use petgraph::prelude::*;
use si_id::{
    AttributeValueId,
    ViewId,
};

use crate::{
    ComponentError,
    ComponentId,
    EdgeWeightKind,
    SchemaVariantId,
    WorkspaceSnapshotError,
    component::ComponentResult,
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants,
        graph::{
            WorkspaceSnapshotGraphResult,
            WorkspaceSnapshotGraphV4,
            traits::component::ComponentExt,
        },
        node_weight::NodeWeightDiscriminants,
    },
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

    pub fn component_contained_in_views(
        &self,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<Vec<ViewId>> {
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

impl ComponentExt for WorkspaceSnapshotGraphV4 {
    fn root_attribute_value(
        &self,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueId> {
        let component_index = self.get_node_index_by_id(component_id)?;
        let root_index = self.target(component_index, EdgeWeightKind::Root)?;
        let root_id = self
            .get_node_weight(root_index)?
            // Make sure it's an AttributeValue before we cast to AttributeValueId
            .get_attribute_value_node_weight()?
            .id()
            .into();
        Ok(root_id)
    }
}
