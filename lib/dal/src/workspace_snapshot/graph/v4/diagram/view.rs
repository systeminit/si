use std::collections::HashSet;

use petgraph::prelude::*;
use si_id::ComponentId;

use crate::{
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    diagram::view::ViewId,
    workspace_snapshot::graph::{
        WorkspaceSnapshotGraphError,
        WorkspaceSnapshotGraphResult,
        WorkspaceSnapshotGraphV4,
        traits::diagram::view::ViewExt,
    },
};

impl ViewExt for WorkspaceSnapshotGraphV4 {
    fn view_remove(&mut self, view_id: ViewId) -> WorkspaceSnapshotGraphResult<()> {
        // If there are any Components remaining in the View, this View _CANNOT_ be the only View they
        // are in. If this View is the only View _ANY_ of the items are in, we do not allow removal
        // of the View.
        let view_node_idx = self.get_node_index_by_id(view_id)?;

        // Do not allow deletion of the default view. We will likely need to get rid of this when
        // we change what views do.
        for (edge_weight, _, _) in self.edges_directed_for_edge_weight_kind(
            view_node_idx,
            Incoming,
            EdgeWeightKindDiscriminants::Use,
        ) {
            if *edge_weight.kind() == EdgeWeightKind::new_use_default() {
                return Err(WorkspaceSnapshotGraphError::DefaultViewDeletionAttempt);
            }
        }

        // View --Use--> Geometry --Represents-->
        //   {Component, DiagramObject <--DiagramObject-- View (on canvas)}
        //
        // Component <--Represents-- Geometry <--Use-- View
        //
        // View (on canvas) --DiagramObject--> DiagramObject <--Represents-- Geometry <--Use-- View
        let mut would_be_orphaned_component_ids = Vec::new();
        for (_edge_weight, _view_node_idx, geometry_node_idx) in self
            .edges_directed_for_edge_weight_kind(
                view_node_idx,
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
        {
            // Find the "thing" this Geometry is representing, so we can make sure it is also
            // represented by at least one Geometry in another View.
            let Some(represented_thing_idx) = self.get_edge_weight_kind_target_idx_opt(
                geometry_node_idx,
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::Represents,
            )?
            else {
                // We found a `Geometry` that doesn't actually "represent" anything, so removing
                // this view can't be orphaning something that doesn't exist.
                continue;
            };
            let represented_thing_node_weight = self.get_node_weight(represented_thing_idx)?;

            if NodeWeightDiscriminants::Component != represented_thing_node_weight.into() {
                // Components _MUST_ be in another View for this View to be able to be removed.
                // Things with DiagramObjects (currently only Views) do not have to be part of
                // another View for this View to be able to be removed.
                continue;
            }

            let mut view_member_idxs = HashSet::new();
            // Find what other views (if any) this Component is a part of.
            for (_edge_weight, geometry_node_idx, _component_idx) in self
                .edges_directed_for_edge_weight_kind(
                    represented_thing_idx,
                    Direction::Incoming,
                    EdgeWeightKindDiscriminants::Represents,
                )
            {
                let geometry_view_idx = self.get_edge_weight_kind_target_idx(
                    geometry_node_idx,
                    Direction::Incoming,
                    EdgeWeightKindDiscriminants::Use,
                )?;
                view_member_idxs.insert(geometry_view_idx);
            }
            view_member_idxs.remove(&view_node_idx);
            if view_member_idxs.is_empty() {
                would_be_orphaned_component_ids.push(represented_thing_node_weight.id());
            }
        }

        if !would_be_orphaned_component_ids.is_empty() {
            return Err(WorkspaceSnapshotGraphError::ViewRemovalWouldOrphanItems(
                would_be_orphaned_component_ids,
            ));
        }

        let mut edge_idxs_to_remove = Vec::new();
        let mut view_geometry_idxs = Vec::new();
        // We need to explicitly remove the DiagramObject for the View as it will have more
        // incoming edges than just the one from the View.
        if let Some(diagram_object_idx) = self.get_edge_weight_kind_target_idx_opt(
            view_node_idx,
            Direction::Outgoing,
            EdgeWeightKindDiscriminants::DiagramObject,
        )? {
            for diagram_object_edgeref in
                self.edges_directed(diagram_object_idx, Direction::Incoming)
            {
                edge_idxs_to_remove.push(diagram_object_edgeref.id());
                if EdgeWeightKindDiscriminants::Represents
                    == diagram_object_edgeref.weight().kind().into()
                {
                    view_geometry_idxs.push(diagram_object_edgeref.source());
                }
            }
        }

        for view_edgeref in self.edges_directed(view_node_idx, Direction::Incoming) {
            edge_idxs_to_remove.push(view_edgeref.id());
        }
        for view_geometry_idx in view_geometry_idxs {
            self.remove_node(view_geometry_idx);
        }
        for edge_idx in edge_idxs_to_remove {
            self.remove_edge_by_idx(edge_idx)?;
        }

        Ok(())
    }

    fn list_for_component_id(
        &self,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<Vec<ViewId>> {
        let mut view_ids_set: HashSet<ViewId> = HashSet::new();

        if let Some(component_idx) = self.get_node_index_by_id_opt(component_id.into_inner()) {
            for (_edge_weight, geometry_node_idx, _component_node_idx) in self
                .edges_directed_for_edge_weight_kind(
                    component_idx,
                    Direction::Incoming,
                    EdgeWeightKindDiscriminants::Represents,
                )
            {
                let view_idx = self.get_edge_weight_kind_target_idx(
                    geometry_node_idx,
                    Direction::Incoming,
                    EdgeWeightKindDiscriminants::Use,
                )?;
                let view_node_weight = self.get_node_weight(view_idx)?;
                view_ids_set.insert(view_node_weight.id().into());
            }
        }

        Ok(view_ids_set.into_iter().collect())
    }
}
