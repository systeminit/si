use petgraph::{
    prelude::*,
    visit::{
        Control,
        DfsEvent,
    },
};
use si_events::workspace_snapshot::EntityKind;
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
        NodeWeight,
        content_address::ContentAddress,
        edge_weight::EdgeWeightKindDiscriminants,
        graph::{
            WorkspaceSnapshotGraphError,
            WorkspaceSnapshotGraphResult,
            WorkspaceSnapshotGraphV4,
            traits::{
                attribute_value::AttributeValueExt,
                component::ComponentExt,
                entity_kind::EntityKindExt,
            },
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

    fn external_source_count_dfs_event(
        event: DfsEvent<NodeIndex>,
        count: &mut usize,
        graph: &Self,
    ) -> WorkspaceSnapshotGraphResult<Control<()>> {
        match event {
            DfsEvent::Discover(node_idx, _) => {
                let av_id = graph
                    .node_index_to_id(node_idx)
                    .ok_or_else(|| WorkspaceSnapshotGraphError::GraphTraversal(event))?;
                let entity_kind = graph.get_entity_kind_for_id(av_id.into())?;

                // Early return if node is not an attribute value
                if !matches!(entity_kind, EntityKind::AttributeValue) {
                    return Ok(petgraph::visit::Control::Prune);
                }

                let Some(prototype_id) = graph.component_prototype_id(av_id.into())? else {
                    return Ok(petgraph::visit::Control::Continue);
                };

                let prototype_node_idx = graph.get_node_index_by_id(prototype_id)?;

                let mut subs_count = 0;

                for (_, _, apa_idx) in graph.edges_directed_for_edge_weight_kind(
                    prototype_node_idx,
                    Direction::Outgoing,
                    EdgeWeightKindDiscriminants::PrototypeArgument,
                ) {
                    subs_count += graph
                        .edges_directed_for_edge_weight_kind(
                            apa_idx,
                            Direction::Outgoing,
                            EdgeWeightKindDiscriminants::ValueSubscription,
                        )
                        .count();
                }

                if subs_count > 0 {
                    *count += subs_count;
                    Ok(petgraph::visit::Control::Prune)
                } else {
                    Ok(petgraph::visit::Control::Continue)
                }
            }
            _ => Ok(petgraph::visit::Control::Continue),
        }
    }
    fn has_socket_connections_dfs_event(
        event: DfsEvent<NodeIndex>,
        has_connections: &mut bool,
        component_id: ComponentId,
        graph: &Self,
    ) -> WorkspaceSnapshotGraphResult<Control<()>> {
        match event {
            DfsEvent::Discover(node_idx, _) => {
                let node_weight = graph.get_node_weight(node_idx)?;

                match node_weight {
                    // Found an APA Node weight - let's see if it has targets and if the component in question is the destination
                    NodeWeight::AttributePrototypeArgument(apa_node_weight) => {
                        if let Some(targets) = apa_node_weight.targets() {
                            if targets.destination_component_id == component_id {
                                *has_connections = true;
                                return Ok(petgraph::visit::Control::Break(()));
                            }
                        }
                        Ok(petgraph::visit::Control::Continue)
                    }
                    NodeWeight::Content(content_node_weight) => {
                        match content_node_weight.content_address() {
                            // we really only care about these I think. The rest can be pruned
                            ContentAddress::AttributePrototype(_)
                            | ContentAddress::InputSocket(_) => {
                                Ok(petgraph::visit::Control::Continue)
                            }

                            _ => Ok(petgraph::visit::Control::Prune),
                        }
                    }
                    // If we see any of these, keep walking
                    NodeWeight::SchemaVariant(_)
                    | NodeWeight::InputSocket(_)
                    | NodeWeight::Prop(_) => Ok(petgraph::visit::Control::Continue),
                    // All of these can be pruned, they don't lead to the answer
                    NodeWeight::Action(_)
                    | NodeWeight::AttributeValue(_)
                    | NodeWeight::Component(_)
                    | NodeWeight::ActionPrototype(_)
                    | NodeWeight::Category(_)
                    | NodeWeight::DependentValueRoot(_)
                    | NodeWeight::FinishedDependentValueRoot(_)
                    | NodeWeight::Ordering(_)
                    | NodeWeight::Geometry(_)
                    | NodeWeight::View(_)
                    | NodeWeight::DiagramObject(_)
                    | NodeWeight::ApprovalRequirementDefinition(_)
                    | NodeWeight::ManagementPrototype(_)
                    | NodeWeight::Func(_)
                    | NodeWeight::FuncArgument(_)
                    | NodeWeight::Reason(_)
                    | NodeWeight::Secret(_) => Ok(petgraph::visit::Control::Prune),
                }
            }
            _ => Ok(petgraph::visit::Control::Continue),
        }
    }
}

impl ComponentExt for WorkspaceSnapshotGraphV4 {
    fn root_attribute_value(&self, component_id: ComponentId) -> ComponentResult<AttributeValueId> {
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

    fn external_source_count(&self, component_id: ComponentId) -> ComponentResult<usize> {
        let root_av_id = self.root_attribute_value(component_id)?;

        let mut count = 0;

        petgraph::visit::depth_first_search(
            self.graph(),
            Some(self.get_node_index_by_id(root_av_id)?),
            |event| Self::external_source_count_dfs_event(event, &mut count, self),
        )?;

        Ok(count)
    }

    fn has_socket_connections(&self, component_id: ComponentId) -> ComponentResult<bool> {
        // Start DFS
        // Path: Component -> SchemaVariant → InputSocket → AttributePrototype (Content) → AttributePrototypeArgument
        let mut has_connections = false;

        let schema_variant_id = self.schema_variant_id_for_component_id(component_id)?;

        petgraph::visit::depth_first_search(
            &self.graph,
            Some(self.get_node_index_by_id(schema_variant_id)?),
            |event| {
                Self::has_socket_connections_dfs_event(
                    event,
                    &mut has_connections,
                    component_id,
                    self,
                )
            },
        )?;

        Ok(has_connections)
    }
}
