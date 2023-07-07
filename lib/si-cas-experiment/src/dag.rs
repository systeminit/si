use std::collections::HashMap;

use petgraph::prelude::*;
use ulid::Ulid;

use crate::{
    change_set::{ChangeSet, ChangeSetPk},
    error::DagError,
    error::DagResult,
    index_and_entry::IndexAndEntry,
    schema::{Schema, SchemaId, SchemaPk},
    vector_clock::VectorClock,
    workspace::{Workspace, WorkspacePk},
};

#[derive(Debug, Clone)]
pub struct SiDag {
    pub graph: StableGraph<SiNode, SiEdge>,
    pub heads: HashMap<ChangeSetPk, WorkspacePk>,
    pub change_sets: HashMap<ChangeSetPk, ChangeSet>,
    pub workspaces: HashMap<WorkspacePk, IndexAndEntry<Workspace>>,
    pub schemas: HashMap<SchemaPk, IndexAndEntry<Schema>>,
    pub vector_clocks: HashMap<Ulid, VectorClock>,
    pub conflicts: HashMap<ChangeSetPk, Vec<Conflict>>,
}

impl SiDag {
    pub fn new(workspace_name: impl Into<String>) -> SiDag {
        let workspace_name = workspace_name.into();
        let mut dag = SiDag {
            heads: HashMap::new(),
            change_sets: HashMap::new(),
            graph: StableGraph::new(),
            workspaces: HashMap::new(),
            schemas: HashMap::new(),
            vector_clocks: HashMap::new(),
            conflicts: HashMap::new(),
        };
        let workspace_pk = dag.create_workspace(workspace_name);
        let workspace = dag.get_workspace(workspace_pk).unwrap();
        let workspace_id = workspace.id();
        let change_set_pk = dag.create_change_set("main", workspace.id(), workspace.pk());
        dag.vector_clocks
            .insert(workspace_pk, VectorClock::new(workspace_id, change_set_pk));
        dag
    }

    pub fn merge_change_set(&mut self, change_set_pk: ChangeSetPk) -> DagResult<()> {
        let conflicts = self.rebase_change_set(change_set_pk)?;
        if !conflicts.is_empty() {
            Err(DagError::MergeHasConflicts(conflicts))
        } else {
            let to_merge_change_set = self.get_change_set_by_pk(change_set_pk)?;
            let to_merge_workspace_pk = self.get_head_for_change_set_pk(change_set_pk)?;
            let target_change_set =
                self.get_change_set_by_name(&to_merge_change_set.target_change_set_name)?;
            self.heads
                .insert(target_change_set.pk(), to_merge_workspace_pk);
            Ok(())
        }
    }

    // TODO: Working on rebase!
    //
    // To rebase, we do the following:
    //  - For each object in the base dag
    //    - Search for objects from the same lineage in the target dag; for each one
    //      - If the target object has seen the base objects vector clock (by being higher than the base objects entry)
    //        - Do nothing.
    //      - If the target object has been edited in this change set
    //        - Mark the object as conflicted and requiring manual resolution
    //      - If the target object is a lower clock than the base object
    //        - Update the target object to the base object
    //        - Update the edges to reflect the edges on the base dag
    pub fn rebase_change_set(&mut self, change_set_pk: ChangeSetPk) -> DagResult<Vec<Conflict>> {
        let change_set = self.get_change_set_by_pk(change_set_pk)?;

        // Get the merge destination workspace
        let destination_workspace_pk =
            self.get_head_for_change_set_name(&change_set.target_change_set_name)?;
        let destination_workspace_idx = self.get_workspace_node_index(destination_workspace_pk)?;

        let mut conflicts = Vec::new();
        let mut updates = Vec::new();
        let mut bfs = Bfs::new(&self.graph, destination_workspace_idx);
        while let Some(node_index) = dbg!(bfs.next(&self.graph)) {
            match dbg!(self
                .graph
                .node_weight(node_index)
                .ok_or(DagError::MissingNodeWeight)?)
            {
                SiNode {
                    kind: SiNodeKind::Workspace(dest_pk),
                } => {
                    for our_object_kind in self
                        .find_all_objects_of_lineage_by_pk_in_change_set(change_set_pk, *dest_pk)?
                    {
                        match our_object_kind {
                            SiObjectKind::Workspace(our_object) => {
                                let ours_is_newer =
                                    self.clock_is_newer(our_object.pk(), *dest_pk)?;
                                // If our object is newer, we can merge our object - just let it happen!
                                if !ours_is_newer {
                                    // If the base is newer than our object
                                    // And we have changed the object in this change set
                                    if self.clock_was_changed_in_changeset(
                                        our_object.pk(),
                                        change_set_pk,
                                    )? {
                                        // We need to notify that this object is a conflict
                                        conflicts.push(Conflict::new(
                                            SiNodeKind::Workspace(*dest_pk),
                                            SiNodeKind::Workspace(our_object.pk()),
                                            change_set_pk,
                                        ));
                                    } else {
                                        // If we have not modified the object in this change set
                                        // We need to update our changeset to have the updated object from the base
                                        updates.push(Update::new(
                                            SiNodeKind::Workspace(*dest_pk),
                                            SiNodeKind::Workspace(our_object.pk()),
                                            false,
                                        ));
                                    }
                                }
                            }
                            _ => return Err(DagError::ObjectMismatch),
                        }
                    }
                }
                SiNode {
                    kind: SiNodeKind::Schema(dest_pk),
                } => {
                    let mut found_lineage = false;
                    for our_object_kind in self
                        .find_all_objects_of_lineage_by_pk_in_change_set(change_set_pk, *dest_pk)?
                    {
                        match our_object_kind {
                            SiObjectKind::Schema(our_object) => {
                                found_lineage = true;
                                let ours_is_newer =
                                    //self.clock_is_newer(our_object.pk(), *dest_pk)?;
                                    dbg!(self.clock_is_newer(dbg!(our_object.pk()), dbg!(*dest_pk))?);
                                // If our object is newer, we can merge our object - just let it happen!
                                if !ours_is_newer {
                                    // And we have changed the object in this change set
                                    if self.clock_was_changed_in_changeset(
                                        our_object.pk(),
                                        change_set_pk,
                                    )? {
                                        // Is the only thing newer about the new object the
                                        // clock for the change set we are in? if so it isn't
                                        // a conflict
                                        if !self.clock_is_newer_for_change_set(
                                            *dest_pk,
                                            our_object.pk(),
                                            change_set_pk,
                                        )? {
                                            // We need to notify that this object is a conflict
                                            conflicts.push(Conflict::new(
                                                SiNodeKind::Schema(*dest_pk),
                                                SiNodeKind::Schema(our_object.pk()),
                                                change_set_pk,
                                            ));
                                        }
                                    } else {
                                        // If we have not modified the object in this change set
                                        // We need to update our changeset to have the updated object from the base
                                        updates.push(Update::new(
                                            SiNodeKind::Schema(*dest_pk),
                                            SiNodeKind::Schema(our_object.pk()),
                                            false,
                                        ));
                                    }
                                }
                            }
                            _ => return Err(DagError::ObjectMismatch),
                        }
                    }
                    if !found_lineage {
                        updates.push(Update::new(
                            SiNodeKind::Schema(*dest_pk),
                            SiNodeKind::Schema(*dest_pk),
                            true,
                        ));
                    }
                }
            }
        }

        // Process updates!
        for update in updates {
            match update.from_object {
                SiNodeKind::Workspace(from_pk) => {
                    if let SiNodeKind::Workspace(mut to_pk) = update.to_object {
                        // Make our change set version a strict superset
                        let from_object = self.get_workspace(from_pk)?.clone();
                        let from_object_pk = from_object.pk();

                        if update.create {
                            to_pk = self.modify_workspace(change_set_pk, |_to_ws| Ok(()))?;
                        } else {
                            to_pk = self.modify_workspace(change_set_pk, |to_ws| {
                                to_ws.name = from_object.name;
                                Ok(())
                            })?;
                        }

                        // Merge the vector clocks
                        self.vector_clock_merge(to_pk, from_pk, change_set_pk)?;

                        // Copy the outgoing edges
                        let from_idx = self.get_workspace_node_index(from_object_pk)?;
                        let to_idx = self.get_workspace_node_index(to_pk)?;
                        let mut to_update = Vec::new();
                        for down_node_idx in
                            self.graph.neighbors_directed(from_idx, Direction::Outgoing)
                        {
                            let existing_edge_idx = self
                                .graph
                                .find_edge(from_idx, down_node_idx)
                                .expect("you had it a minute ago");
                            let existing_edge_weight = self
                                .graph
                                .edge_weight(existing_edge_idx)
                                .expect("you def have an edge weight");
                            let new_edge_weight = existing_edge_weight.clone();
                            to_update.push((to_idx, down_node_idx, new_edge_weight));
                        }
                        for (to_idx, down_node_idx, new_edge_weight) in to_update.into_iter() {
                            self.graph
                                .update_edge(to_idx, down_node_idx, new_edge_weight);
                        }
                    } else {
                        return Err(DagError::MismatchedUpdateObject);
                    }
                }
                SiNodeKind::Schema(from_pk) => {
                    if let SiNodeKind::Schema(mut to_pk) = update.to_object {
                        // Make our change set version a strict superset
                        let from_object = self.get_schema_by_pk(from_pk)?.clone();
                        let from_object_pk = from_object.pk();

                        if update.create {
                            to_pk = self.modify_schema(change_set_pk, to_pk, |_to_s| Ok(()))?;
                        } else {
                            to_pk = self.modify_schema(change_set_pk, to_pk, |to_s| {
                                to_s.name = from_object.name;
                                Ok(())
                            })?;
                        }

                        // Merge the vector clocks
                        self.vector_clock_merge(to_pk, from_pk, change_set_pk)?;

                        //// Copy the outgoing edges
                        let from_idx = self.get_schema_node_index(from_object_pk)?;
                        let to_idx = self.get_schema_node_index(to_pk)?;
                        let mut to_update = Vec::new();
                        for down_node_idx in
                            self.graph.neighbors_directed(from_idx, Direction::Outgoing)
                        {
                            let existing_edge_idx = self
                                .graph
                                .find_edge(from_idx, down_node_idx)
                                .expect("you had it a minute ago");
                            let existing_edge_weight = self
                                .graph
                                .edge_weight(existing_edge_idx)
                                .expect("you def have an edge weight");
                            let new_edge_weight = existing_edge_weight.clone();
                            to_update.push((to_idx, down_node_idx, new_edge_weight));
                        }
                        for (to_idx, down_node_idx, new_edge_weight) in to_update.into_iter() {
                            self.graph
                                .update_edge(to_idx, down_node_idx, new_edge_weight);
                        }

                        //// Copy the incoming edges
                        let mut to_update = Vec::new();
                        for up_node_idx in
                            self.graph.neighbors_directed(from_idx, Direction::Incoming)
                        {
                            let existing_edge_idx = self
                                .graph
                                .find_edge(up_node_idx, from_idx)
                                .expect("you had it a minute ago");
                            let existing_edge_weight = self
                                .graph
                                .edge_weight(existing_edge_idx)
                                .expect("you def have an edge weight");
                            let new_edge_weight = existing_edge_weight.clone();
                            to_update.push((up_node_idx, to_idx, new_edge_weight));
                        }
                        for (up_node_idx, to_idx, new_edge_weight) in to_update.into_iter() {
                            self.graph.update_edge(up_node_idx, to_idx, new_edge_weight);
                        }
                    } else {
                        return Err(DagError::MismatchedUpdateObject);
                    }
                }
            }
        }

        if !conflicts.is_empty() {
            self.conflicts.insert(change_set_pk, conflicts.clone());
        } else {
            self.conflicts.remove(&change_set_pk);
        }

        // Return any conflicts
        Ok(conflicts)
    }

    pub fn all_objects_in_head_for_change_set_name(
        &self,
        name: impl Into<String>,
    ) -> DagResult<Vec<SiObjectKind>> {
        let name = name.into();
        let workspace_pk = self.get_head_for_change_set_name(name)?;
        let workspace_idx = self.get_workspace_node_index(workspace_pk)?;
        let mut found = Vec::new();
        let mut bfs = Bfs::new(&self.graph, workspace_idx);
        while let Some(node_index) = bfs.next(&self.graph) {
            match self
                .graph
                .node_weight(node_index)
                .ok_or(DagError::MissingNodeWeight)?
            {
                SiNode {
                    kind: SiNodeKind::Workspace(pk),
                } => {
                    let o = self.get_workspace(*pk)?;
                    found.push(SiObjectKind::Workspace(o.clone()));
                }
                SiNode {
                    kind: SiNodeKind::Schema(pk),
                } => {
                    let o = self.get_schema_by_pk(*pk)?;
                    found.push(SiObjectKind::Schema(o.clone()));
                }
            }
        }
        Ok(found)
    }

    pub fn find_all_objects_of_lineage_by_pk_in_change_set(
        &self,
        change_set_pk: ChangeSetPk,
        object_pk: Ulid,
    ) -> DagResult<Vec<SiObjectKind>> {
        if let Ok(workspace) = self.get_workspace(object_pk) {
            return self
                .find_all_objects_of_lineage_by_id_in_change_set(change_set_pk, workspace.id());
        }
        if let Ok(schema) = self.get_schema_by_pk(object_pk) {
            return self
                .find_all_objects_of_lineage_by_id_in_change_set(change_set_pk, schema.id());
        }
        Err(DagError::CannotFindObjectByPk)
    }

    // Super inefficient, but - prototypes!
    pub fn find_all_objects_of_lineage_by_id_in_change_set(
        &self,
        change_set_pk: ChangeSetPk,
        object_id: Ulid,
    ) -> DagResult<Vec<SiObjectKind>> {
        let workspace_pk = self.get_head_for_change_set_pk(change_set_pk)?;
        let workspace_idx = self.get_workspace_node_index(workspace_pk)?;

        let mut found = Vec::new();
        let mut bfs = Bfs::new(&self.graph, workspace_idx);
        while let Some(node_index) = bfs.next(&self.graph) {
            match self
                .graph
                .node_weight(node_index)
                .ok_or(DagError::MissingNodeWeight)?
            {
                SiNode {
                    kind: SiNodeKind::Workspace(pk),
                } => {
                    let o = self.get_workspace(*pk)?;
                    if o.id() == object_id {
                        found.push(SiObjectKind::Workspace(o.clone()));
                    }
                }
                SiNode {
                    kind: SiNodeKind::Schema(pk),
                } => {
                    let o = self.get_schema_by_pk(*pk)?;
                    if o.id() == object_id {
                        found.push(SiObjectKind::Schema(o.clone()));
                    }
                }
            }
        }
        Ok(found)
    }

    pub fn create_change_set(
        &mut self,
        name: impl Into<String>,
        target_change_set_name: impl Into<String>,
        workspace_pk: WorkspacePk,
    ) -> ChangeSetPk {
        let name = name.into();
        let target_change_set_name = target_change_set_name.into();
        let change_set = ChangeSet::new(name, target_change_set_name, workspace_pk);
        let change_set_pk = change_set.pk();
        self.change_sets.insert(change_set.pk(), change_set);
        self.heads.insert(change_set_pk, workspace_pk);
        change_set_pk
    }

    pub fn get_change_set_by_pk(&self, change_set_pk: ChangeSetPk) -> DagResult<&ChangeSet> {
        self.change_sets
            .get(&change_set_pk)
            .ok_or(DagError::ChangeSetNotFound(change_set_pk))
    }

    pub fn get_change_set_by_name(&self, name: impl Into<String>) -> DagResult<&ChangeSet> {
        let name = name.into();
        self.change_sets
            .values()
            .find(|cs| cs.name == name)
            .ok_or(DagError::ChangeSetNameNotFound(name))
    }

    pub fn get_head_for_change_set_pk(&self, change_set_pk: ChangeSetPk) -> DagResult<WorkspacePk> {
        Ok(*self
            .heads
            .get(&change_set_pk)
            .ok_or(DagError::ChangeSetNotFound(change_set_pk))?)
    }

    pub fn get_head_for_change_set_name(&self, name: impl Into<String>) -> DagResult<WorkspacePk> {
        let name = name.into();
        let change_set = self
            .change_sets
            .values()
            .find(|cs| cs.name == name)
            .ok_or(DagError::ChangeSetNameNotFound(name.clone()))?;
        let workspace_pk = self
            .heads
            .get(&change_set.pk())
            .ok_or(DagError::ChangeSetNotFound(change_set.pk()))?;
        Ok(*workspace_pk)
    }

    pub fn modify_workspace<L>(
        &mut self,
        change_set_pk: ChangeSetPk,
        lambda: L,
    ) -> DagResult<WorkspacePk>
    where
        L: FnOnce(&mut Workspace) -> DagResult<()>,
    {
        let workspace_pk = self.get_head_for_change_set_pk(change_set_pk)?;
        let base_object = self.get_workspace(workspace_pk)?;
        let base_index = self.get_workspace_node_index(workspace_pk)?;
        let new_vector_clock = self
            .vector_clocks
            .get(&workspace_pk)
            .ok_or(DagError::VectorClockNotFound)?
            .clone();

        let mut new_workspace = base_object.clone();
        new_workspace.pk = WorkspacePk::new();
        let new_workspace_pk = new_workspace.pk();
        let new_index = self
            .graph
            .add_node(SiNode::new(SiNodeKind::Workspace(new_workspace.pk())));
        self.vector_clocks
            .insert(new_workspace_pk, new_vector_clock);

        // Anything you want to do to the workspace happens here
        lambda(&mut new_workspace)?;

        self.workspaces.insert(
            new_workspace.pk(),
            IndexAndEntry::new(new_index, new_workspace),
        );

        let mut edges_to_make: Vec<NodeIndex> = Vec::new();
        for node_idx in self
            .graph
            .neighbors_directed(base_index, Direction::Outgoing)
        {
            edges_to_make.push(node_idx);
        }
        for node_idx in edges_to_make {
            self.graph
                .add_edge(new_index, node_idx, SiEdge::new(SiEdgeKind::Uses));
        }

        self.update_workspace_content_hash(new_workspace_pk)?;

        self.vector_clock_increment(new_workspace_pk, change_set_pk);

        self.update_head(change_set_pk, new_workspace_pk);

        Ok(new_workspace_pk)
    }

    pub fn workspace_replace_schema(
        &mut self,
        change_set_pk: ChangeSetPk,
        old_schema_pk: SchemaPk,
        new_schema_pk: SchemaPk,
    ) -> DagResult<WorkspacePk> {
        let workspace_pk = self.get_head_for_change_set_pk(change_set_pk)?;
        let base_object = self.get_workspace(workspace_pk)?;
        let base_index = self.get_workspace_node_index(workspace_pk)?;
        let new_vector_clock = self
            .vector_clocks
            .get(&workspace_pk)
            .ok_or(DagError::VectorClockNotFound)?
            .clone();

        let mut new_workspace = base_object.clone();
        new_workspace.pk = WorkspacePk::new();
        let new_workspace_pk = new_workspace.pk();
        let new_index = self
            .graph
            .add_node(SiNode::new(SiNodeKind::Workspace(new_workspace.pk())));
        self.vector_clocks
            .insert(new_workspace_pk, new_vector_clock);

        self.workspaces.insert(
            new_workspace.pk(),
            IndexAndEntry::new(new_index, new_workspace),
        );

        let old_schema_node_index = self
            .schemas
            .get(&old_schema_pk)
            .ok_or(DagError::SchemaNotFound(old_schema_pk))?
            .node_index;
        let new_schema_node_index = self
            .schemas
            .get(&new_schema_pk)
            .ok_or(DagError::SchemaNotFound(new_schema_pk))?
            .node_index;
        let mut edges_to_make = vec![new_schema_node_index];
        for node_idx in self
            .graph
            .neighbors_directed(base_index, Direction::Outgoing)
        {
            // We don't want to carry over the edge to the old "version" of the schema.
            if node_idx == old_schema_node_index {
                continue;
            }
            edges_to_make.push(node_idx);
        }
        for node_idx in edges_to_make {
            self.graph
                .add_edge(new_index, node_idx, SiEdge::new(SiEdgeKind::Uses));
        }

        self.update_workspace_content_hash(new_workspace_pk)?;

        self.vector_clock_increment(new_workspace_pk, change_set_pk);

        self.update_head(change_set_pk, new_workspace_pk);

        Ok(new_workspace_pk)
    }

    // TODO: This should really error if the schema isn't reachable from the provided change_set/workspace.
    pub fn modify_schema<L>(
        &mut self,
        change_set_pk: ChangeSetPk,
        schema_pk: SchemaPk,
        lambda: L,
    ) -> DagResult<SchemaPk>
    where
        L: FnOnce(&mut Schema) -> DagResult<()>,
    {
        let base_object = self.get_schema_by_pk(schema_pk)?;
        let base_index = self.get_schema_node_index(schema_pk)?;
        let new_vector_clock = self
            .vector_clocks
            .get(&schema_pk)
            .ok_or(DagError::VectorClockNotFound)?
            .clone();

        let mut new_schema = base_object.clone();
        new_schema.pk = SchemaPk::new();
        let new_schema_pk = new_schema.pk();
        let new_index = self
            .graph
            .add_node(SiNode::new(SiNodeKind::Schema(new_schema.pk())));
        self.vector_clocks.insert(new_schema_pk, new_vector_clock);
        println!("{:?}", petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel]));

        // Anything you want to do to the schema happens here
        lambda(&mut new_schema)?;

        self.schemas
            .insert(new_schema.pk(), IndexAndEntry::new(new_index, new_schema));

        // Outgoing edges are things we depend on. We do not affect their content hash.
        let mut edges_to_make: Vec<NodeIndex> = Vec::new();
        for node_idx in self
            .graph
            .neighbors_directed(base_index, Direction::Outgoing)
        {
            edges_to_make.push(node_idx);
        }
        for node_idx in edges_to_make {
            self.graph
                .update_edge(new_index, node_idx, SiEdge::new(SiEdgeKind::Uses));
        }
        println!("{:?}", petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel]));

        // Incoming edges need a new "thing" created to reference us, since outgoing edges are
        // considered to be part of the "thing" itself (since they affect the things content hash).

        self.update_schema_content_hash(new_schema_pk)?;
        self.workspace_replace_schema(change_set_pk, schema_pk, new_schema_pk)?;

        self.vector_clock_increment(new_schema_pk, change_set_pk);

        Ok(new_schema_pk)
    }

    pub fn update_head(&mut self, change_set_pk: ChangeSetPk, workspace_pk: WorkspacePk) {
        self.heads.insert(change_set_pk, workspace_pk);
    }

    pub fn vector_clock_increment(&mut self, object_pk: Ulid, change_set_pk: ChangeSetPk) {
        match self.vector_clocks.get_mut(&object_pk) {
            Some(vc) => vc.inc(change_set_pk),
            None => {
                if let Ok(o) = self.get_workspace(object_pk) {
                    self.vector_clocks
                        .insert(object_pk, VectorClock::new(o.id(), change_set_pk));
                    return;
                }
                if let Ok(o) = self.get_schema_by_pk(object_pk) {
                    self.vector_clocks
                        .insert(object_pk, VectorClock::new(o.id(), change_set_pk));
                }
            }
        }
    }

    pub fn is_node_index_in_change_set(
        &self,
        change_set_pk: ChangeSetPk,
        node_index: NodeIndex,
    ) -> DagResult<bool> {
        let workspace_node_index =
            self.get_workspace_node_index(self.get_head_for_change_set_pk(change_set_pk)?)?;
        // A* may not be the best algorithm to find out "Is this node reachable from the root we're talking about?",
        // but it's reasonably fast, and very well understood.
        Ok(petgraph::algo::astar(
            &self.graph,
            workspace_node_index,
            |node| node == node_index,
            |_| 1,
            |_| 0,
        )
        .is_some())
    }

    pub fn vector_clock_merge(
        &mut self,
        left_pk: Ulid,
        right_pk: Ulid,
        change_set_pk: ChangeSetPk,
    ) -> DagResult<()> {
        let mut left_vc = self
            .vector_clocks
            .get_mut(&left_pk)
            .ok_or(DagError::VectorClockNotFound)?
            .clone();
        let right_vc = self
            .vector_clocks
            .get(&right_pk)
            .ok_or(DagError::VectorClockNotFound)?;
        left_vc.merge(change_set_pk, right_vc)?;
        self.vector_clocks.insert(left_pk, left_vc);

        Ok(())
    }

    pub fn create_workspace(&mut self, name: impl Into<String>) -> WorkspacePk {
        let workspace = Workspace::new(name);
        let node = SiNode::new(SiNodeKind::Workspace(workspace.pk()));
        let node_index = self.graph.add_node(node);
        let workspace_pk = workspace.pk();
        self.workspaces
            .insert(workspace.pk(), IndexAndEntry::new(node_index, workspace));
        workspace_pk
    }

    pub fn get_workspace(&self, workspace_pk: WorkspacePk) -> DagResult<&Workspace> {
        self.workspaces
            .get(&workspace_pk)
            .map(|e| e.entry())
            .ok_or(DagError::WorkspaceNotFound(workspace_pk))
    }

    pub fn get_workspace_node_index(&self, workspace_pk: WorkspacePk) -> DagResult<NodeIndex> {
        self.workspaces
            .get(&workspace_pk)
            .map(|e| e.node_index())
            .ok_or(DagError::WorkspaceNotFound(workspace_pk))
    }

    // Through change set
    pub fn create_schema(
        &mut self,
        change_set_pk: ChangeSetPk,
        name: impl Into<String>,
    ) -> DagResult<SchemaPk> {
        //let workspace_pk = self.get_head_for_change_set_pk(change_set_pk)?;
        let workspace_pk = self.modify_workspace(change_set_pk, |_| Ok(()))?;
        let workspace_index = self.get_workspace_node_index(workspace_pk)?;

        let schema = Schema::new(name);
        let schema_id = schema.id();
        let node = SiNode::new(SiNodeKind::Schema(schema.pk()));
        let node_index = self.graph.add_node(node);
        let schema_pk = schema.pk();
        self.schemas
            .insert(schema.pk(), IndexAndEntry::new(node_index, schema));

        let edge = SiEdge::new(SiEdgeKind::Uses);
        self.graph.update_edge(workspace_index, node_index, edge);

        self.update_schema_content_hash(schema_pk)?;

        self.vector_clocks
            .entry(schema_pk)
            .and_modify(|vc| vc.inc(change_set_pk))
            .or_insert(VectorClock::new(schema_id, change_set_pk));

        Ok(schema_pk)
    }

    pub fn get_schema_by_pk(&self, pk: SchemaPk) -> DagResult<&Schema> {
        self.schemas
            .get(&pk)
            .map(|e| e.entry())
            .ok_or(DagError::SchemaNotFound(pk))
    }

    pub fn get_schema_node_index(&self, pk: SchemaPk) -> DagResult<NodeIndex> {
        self.schemas
            .get(&pk)
            .map(|e| e.node_index())
            .ok_or(DagError::SchemaNotFound(pk))
    }

    // Through change set
    pub fn get_workspace_schemas(&self, change_set_pk: ChangeSetPk) -> DagResult<Vec<&Schema>> {
        let workspace_pk = self.get_head_for_change_set_pk(change_set_pk)?;
        let workspace_index = self.get_workspace_node_index(workspace_pk)?;
        let mut schemas = Vec::new();
        for node_idx in self
            .graph
            .neighbors_directed(workspace_index, Direction::Outgoing)
        {
            match self.graph.node_weight(node_idx) {
                Some(SiNode {
                    kind: SiNodeKind::Schema(schema_pk),
                }) => {
                    let schema = self.get_schema_by_pk(*schema_pk)?;
                    schemas.push(schema)
                }
                _ => continue,
            }
        }
        Ok(schemas)
    }

    pub fn set_schema_content_hash(
        &mut self,
        schema_id: SchemaId,
        content_hash: blake3::Hash,
    ) -> DagResult<()> {
        let schema = self
            .schemas
            .get_mut(&schema_id)
            .ok_or(DagError::SchemaNotFound(schema_id))?;
        schema.entry.content_hash = content_hash;
        Ok(())
    }

    pub fn set_workspace_content_hash(
        &mut self,
        id: WorkspacePk,
        content_hash: blake3::Hash,
    ) -> DagResult<()> {
        let object = self
            .workspaces
            .get_mut(&id)
            .ok_or(DagError::WorkspaceNotFound(id))?;
        object.entry.content_hash = content_hash;
        Ok(())
    }

    pub fn update_schema_content_hash(&mut self, schema_pk: SchemaPk) -> DagResult<()> {
        let object = self.get_schema_by_pk(schema_pk)?;
        let index = self.get_schema_node_index(schema_pk)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(object.name.as_bytes());
        hasher.update(object.origin_id.to_string().as_bytes());

        // What I need to calculate my own content hash
        for node_idx in self.graph.neighbors_directed(index, Direction::Outgoing) {
            match self.graph.node_weight(node_idx) {
                // Should have the extraction for each type of thing that a schema depends on
                _ => continue,
            }
        }

        self.set_schema_content_hash(schema_pk, hasher.finalize())?;

        // Trigger recalulating the other content hashes
        for node_idx in self.graph.neighbors_directed(index, Direction::Incoming) {
            match self.graph.node_weight(node_idx) {
                Some(SiNode {
                    kind: SiNodeKind::Workspace(_),
                }) => {
                    // Knowing which workspace to update, and updating it is handled
                    // by the actual "modification" of the schema. Nothing to do here.
                }
                incoming_node => unimplemented!("Updating edge for {incoming_node:?} that depends on a schema ({schema_pk:?}) is not implemented."),
            }
        }

        Ok(())
    }

    pub fn update_workspace_content_hash(&mut self, workspace_pk: WorkspacePk) -> DagResult<()> {
        let object = self.get_workspace(workspace_pk)?;
        let index = self.get_workspace_node_index(workspace_pk)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(object.name.as_bytes());
        hasher.update(object.origin_id.to_string().as_bytes());

        // What I need to calculate my own content hash
        for node_idx in self.graph.neighbors_directed(index, Direction::Outgoing) {
            match self.graph.node_weight(node_idx) {
                Some(SiNode {
                    kind: SiNodeKind::Schema(schema_pk),
                }) => {
                    let schema = self.get_schema_by_pk(*schema_pk)?;
                    hasher.update(schema.content_hash.as_bytes());
                }
                // Should have the extraction for each type of thing that a schema depends on
                _ => continue,
            }
        }

        self.set_workspace_content_hash(workspace_pk, hasher.finalize())?;

        Ok(())
    }

    // A clock is newer if it has seen all of the right clocks entries
    pub fn clock_is_newer(&self, left: Ulid, right: Ulid) -> DagResult<bool> {
        let left_vc = self
            .vector_clocks
            .get(&left)
            .ok_or(DagError::VectorClockNotFound)?;
        let right_vc = self
            .vector_clocks
            .get(&right)
            .ok_or(DagError::VectorClockNotFound)?;
        Ok(left_vc.is_newer(right_vc))
    }

    pub fn clock_is_newer_for_change_set(
        &self,
        left: Ulid,
        right: Ulid,
        change_set_pk: ChangeSetPk,
    ) -> DagResult<bool> {
        let left_vc = self
            .vector_clocks
            .get(&left)
            .ok_or(DagError::VectorClockNotFound)?;
        let right_vc = self
            .vector_clocks
            .get(&right)
            .ok_or(DagError::VectorClockNotFound)?;
        Ok(left_vc.newer_for_change_set(change_set_pk, right_vc))
    }

    pub fn clock_was_changed_in_changeset(
        &self,
        object_id: Ulid,
        change_set_pk: ChangeSetPk,
    ) -> DagResult<bool> {
        let vc = self
            .vector_clocks
            .get(&object_id)
            .ok_or(DagError::VectorClockNotFound)?;
        Ok(vc.was_changed_in_changeset(change_set_pk))
    }

    pub fn resolve_conflict(
        &mut self,
        winner_pk: Ulid,
        loser_pk: Ulid,
        change_set_pk: ChangeSetPk,
    ) -> DagResult<()> {
        self.vector_clock_merge(winner_pk, loser_pk, change_set_pk)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub dest_object: SiNodeKind,
    pub our_object: SiNodeKind,
    pub change_set_pk: ChangeSetPk,
}

impl Conflict {
    pub fn new(
        dest_object: SiNodeKind,
        our_object: SiNodeKind,
        change_set_pk: ChangeSetPk,
    ) -> Conflict {
        Conflict {
            dest_object,
            our_object,
            change_set_pk,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Update {
    pub from_object: SiNodeKind,
    pub to_object: SiNodeKind,
    pub create: bool,
}

impl Update {
    pub fn new(from_object: SiNodeKind, to_object: SiNodeKind, create: bool) -> Update {
        Update {
            from_object,
            to_object,
            create,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SiNodeKind {
    Workspace(WorkspacePk),
    Schema(SchemaPk),
}

#[derive(Debug, Clone)]
pub struct SiNode {
    pub kind: SiNodeKind,
}

impl SiNode {
    pub fn new(kind: SiNodeKind) -> SiNode {
        SiNode { kind }
    }
}

#[derive(Debug, Clone)]
pub enum SiEdgeKind {
    Uses,
}

pub type SiEdgeId = Ulid;
#[derive(Debug, Clone)]
pub struct SiEdge {
    pub kind: SiEdgeKind,
    pub id: SiEdgeId,
}

impl SiEdge {
    pub fn new(kind: SiEdgeKind) -> SiEdge {
        let id = SiEdgeId::new();
        SiEdge { id, kind }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SiObjectKind {
    Workspace(Workspace),
    Schema(Schema),
}

impl SiObjectKind {
    pub fn id(&self) -> Ulid {
        match self {
            SiObjectKind::Workspace(w) => w.id(),
            SiObjectKind::Schema(s) => s.id(),
        }
    }

    pub fn pk(&self) -> Ulid {
        match self {
            SiObjectKind::Workspace(w) => w.pk(),
            SiObjectKind::Schema(s) => s.pk(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_dag() {
        let dag = SiDag::new("poop");
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        let workspace = dag.get_workspace(head_workspace_pk).unwrap();
        assert_eq!(workspace.name, "poop");
        let workspace_vector_clock = dag.vector_clocks.get(&workspace.pk()).unwrap();
        assert_eq!(workspace_vector_clock.clock_entries.len(), 1);
    }

    #[test]
    fn create_new_change_set() {
        let mut dag = SiDag::new("poop");
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        let change_set_pk = dag.create_change_set("killswitch", "main", head_workspace_pk);
        let change_set = dag.get_change_set_by_name("killswitch").unwrap();
        assert_eq!(change_set.pk, change_set_pk);
        assert_eq!(change_set.name, "killswitch");
    }

    #[test]
    fn add_schema_to_workspace_in_a_change_set() {
        // Create a dag
        let mut dag = SiDag::new("poop");

        // Get the head workspace
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();

        // Create a change set
        let change_set_pk = dag.create_change_set("killswitch", "main", head_workspace_pk);

        // Save the original workspace content hash
        let killswitch_workspace_pk = dag.get_head_for_change_set_pk(change_set_pk).unwrap();
        let killswitch_workspace_og = dag.get_workspace(killswitch_workspace_pk).unwrap();
        let og_id = killswitch_workspace_og.id();
        let og_pk = killswitch_workspace_og.pk();
        let og_hash = killswitch_workspace_og.content_hash;

        // Add a schema to the change set
        let schema_pk = dag.create_schema(change_set_pk, "jesse leach").unwrap();

        // Confirm the two workspaces are different
        let new_killswitch_workspace_pk = dag.get_head_for_change_set_pk(change_set_pk).unwrap();
        let new_killswitch_workspace = dag.get_workspace(new_killswitch_workspace_pk).unwrap();
        assert_eq!(og_id, new_killswitch_workspace.id());
        assert_ne!(og_pk, new_killswitch_workspace.pk());
        assert_ne!(og_hash, new_killswitch_workspace.content_hash);

        // Assert the schema shows up if we are in the change set, but not on main
        let main_change_set = dag.get_change_set_by_name("main").unwrap();
        let main_schemas = dag.get_workspace_schemas(main_change_set.pk()).unwrap();
        assert_eq!(main_schemas.len(), 0);
        let killswitch_schemas = dag.get_workspace_schemas(change_set_pk).unwrap();
        assert_eq!(killswitch_schemas.len(), 1);
        assert_eq!(killswitch_schemas[0].pk(), schema_pk);
    }

    #[test]
    fn fast_forward_merge_to_main() {
        // Create a new dag
        let mut dag = SiDag::new("poop");

        // Get the head workspace
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();

        // Create a change set
        let change_set_pk = dag.create_change_set("killswitch", "main", head_workspace_pk);

        // Modify the workspace
        let modified_workspace_pk = dag
            .modify_workspace(change_set_pk, |w| {
                w.name = "back to the future".to_string();
                Ok(())
            })
            .unwrap();

        dag.merge_change_set(change_set_pk).unwrap();

        assert_ne!(head_workspace_pk, modified_workspace_pk);

        let new_head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        assert_eq!(modified_workspace_pk, new_head_workspace_pk);
    }

    #[test]
    fn rebase_simple_fast_forward() {
        // Create a new dag
        let mut dag = SiDag::new("funtimes");
        let main_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        let main_workspace = dag.get_workspace(main_workspace_pk).unwrap();
        assert_eq!(main_workspace.name, "funtimes");

        // Get the head workspace
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();

        // Create three change sets
        let killswitch_change_set_pk =
            dag.create_change_set("killswitch", "main", head_workspace_pk);
        let slayer_change_set_pk = dag.create_change_set("slayer", "main", head_workspace_pk);
        let etid_change_set_pk = dag.create_change_set("etid", "main", head_workspace_pk);
        // Modify the workspace in the etid change set
        let _modified_workspace_pk = dag
            .modify_workspace(etid_change_set_pk, |w| {
                w.name = "radical".to_string();
                Ok(())
            })
            .unwrap();

        // Modify the workspace in the killswitch change set
        let _modified_workspace_pk = dag
            .modify_workspace(killswitch_change_set_pk, |w| {
                w.name = "serenade".to_string();
                Ok(())
            })
            .unwrap();

        // We can merge killswitch because it is a fast forward to main
        dag.merge_change_set(killswitch_change_set_pk).unwrap();

        let main_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        let main_workspace = dag.get_workspace(main_workspace_pk).unwrap();
        assert_eq!(main_workspace.name, "serenade");

        // We can merge slayer because it is a fast forward when the auto-rebase kicks in - its older than
        // killswitch, and it hasn't had any local changes. So it can go.
        dag.merge_change_set(slayer_change_set_pk).unwrap();

        // We are still serenade
        let main_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        let main_workspace = dag.get_workspace(main_workspace_pk).unwrap();
        assert_eq!(main_workspace.name, "serenade");

        // We cannot merge etid, because main has moved on and we haven't seen the changes
        if let Err(DagError::MergeHasConflicts(conflicts)) =
            dag.merge_change_set(etid_change_set_pk)
        {
            let conflict = &conflicts[0];
            if let SiNodeKind::Workspace(our_pk) = conflict.our_object {
                if let SiNodeKind::Workspace(dest_pk) = conflict.dest_object {
                    dag.resolve_conflict(our_pk, dest_pk, etid_change_set_pk)
                        .unwrap();
                }
            }
        } else {
            panic!("etid merged without conflict, but it should have conflicted");
        }

        // We can now merge cleanly, as we resolved the conflict
        dag.merge_change_set(etid_change_set_pk).unwrap();
        let main_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        let main_workspace = dag.get_workspace(main_workspace_pk).unwrap();
        assert_eq!(main_workspace.name, "radical");
    }

    #[test]
    fn find_all_objects_in_change_set() {
        // Create a new dag
        let mut dag = SiDag::new("funtimes");

        // Get the head workspace
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();

        // Create a change sets
        let killswitch_change_set_pk =
            dag.create_change_set("killswitch", "main", head_workspace_pk);

        // Add a couple schema to the workspace
        let audioslave_pk = dag
            .create_schema(killswitch_change_set_pk, "audioslave")
            .unwrap();
        let soundgarden_pk = dag
            .create_schema(killswitch_change_set_pk, "soundgarden")
            .unwrap();

        // Search for soundgarden via its pk
        let search_result = dag
            .find_all_objects_of_lineage_by_pk_in_change_set(
                killswitch_change_set_pk,
                soundgarden_pk,
            )
            .unwrap();
        assert_eq!(search_result.len(), 1);
        match &search_result[0] {
            SiObjectKind::Schema(sg) => {
                assert!(sg.pk() == soundgarden_pk);
            }
            _ => panic!("got a different kind of object then we searched for"),
        }

        // Search for audioslave via its id
        let audioslave = dag.get_schema_by_pk(audioslave_pk).unwrap();
        let search_result = dag
            .find_all_objects_of_lineage_by_id_in_change_set(
                killswitch_change_set_pk,
                audioslave.id(),
            )
            .unwrap();
        assert_eq!(search_result.len(), 1);
        assert_eq!(search_result[0], SiObjectKind::Schema(audioslave.clone()));
    }

    #[test]
    fn merge_complex_change_set() {
        // Create a new dag
        let mut dag = SiDag::new("funtimes");

        // Get the head workspace
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();

        // Create three change sets
        let _killswitch_change_set_pk =
            dag.create_change_set("killswitch", "main", head_workspace_pk);
        let slayer_change_set_pk = dag.create_change_set("slayer", "main", head_workspace_pk);
        let boygenius_change_set_pk = dag.create_change_set("boygenius", "main", head_workspace_pk);

        // Modify the workspace in the boygenius change set
        // Add some schemas to the workspace, then merge it, and confirm we have everything on main
        let _modified_workspace_pk = dag
            .modify_workspace(boygenius_change_set_pk, |w| {
                w.name = "boygenius".to_string();
                Ok(())
            })
            .unwrap();

        let phoebe_schema_pk = dag
            .create_schema(boygenius_change_set_pk, "phoebe")
            .unwrap();
        let julien_schema_pk = dag
            .create_schema(boygenius_change_set_pk, "julien")
            .unwrap();
        let lucy_schema_pk = dag.create_schema(boygenius_change_set_pk, "lucy").unwrap();
        // Should merge cleanly; change set has not modified any existing data.
        dag.merge_change_set(boygenius_change_set_pk).unwrap();

        let main_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();
        let objects = dag.all_objects_in_head_for_change_set_name("main").unwrap();
        assert!(objects.iter().any(|o| o.pk() == main_workspace_pk));
        assert!(objects.iter().any(|o| o.pk() == phoebe_schema_pk));
        assert!(objects.iter().any(|o| o.pk() == julien_schema_pk));
        assert!(objects.iter().any(|o| o.pk() == lucy_schema_pk));

        // Rebase slayer on main, and confirm the objects are all there
        let conflicts = dag.rebase_change_set(slayer_change_set_pk).unwrap();
        // Should be a fast-forward, since this change set hasn't changed anything since it was created.
        assert!(conflicts.is_empty());
        let objects = dag
            .all_objects_in_head_for_change_set_name("slayer")
            .unwrap();
        println!("{:?}", petgraph::dot::Dot::with_config(&dag.graph, &[petgraph::dot::Config::EdgeNoLabel]));
        let slayer_workspace_pk = dbg!(dag.get_head_for_change_set_name("slayer").unwrap());
        dbg!(dag.get_head_for_change_set_name("main").unwrap());
        assert!(objects.iter().any(|o| o.pk() == slayer_workspace_pk));
        assert!(objects.iter().any(|o| o.pk() == phoebe_schema_pk));
        assert!(objects.iter().any(|o| o.pk() == julien_schema_pk));
        assert!(objects.iter().any(|o| o.pk() == lucy_schema_pk));

        let _main_phoebe_clock = dbg!(dag.vector_clocks.get(&phoebe_schema_pk).unwrap());

        // Change the name of a schema on slayer
        let new_phoebe_schema_pk = dag
            .modify_schema(slayer_change_set_pk, phoebe_schema_pk, |s| {
                s.name = "jennifer".to_string();
                Ok(())
            })
            .unwrap();
        let main_phoebe_clock = dbg!(dag.vector_clocks.get(&phoebe_schema_pk).unwrap());
        let new_phoebe_clock = dbg!(dag.vector_clocks.get(&new_phoebe_schema_pk).unwrap());
        dbg!(new_phoebe_clock.is_newer(main_phoebe_clock));

        // Merge slayer - it's clean!
        dag.merge_change_set(slayer_change_set_pk).unwrap();
    }

    #[test]
    fn changes_are_only_reachable_from_the_change_set_that_made_them_until_merged() {
        // Create DAG
        let mut dag = SiDag::new("change sets compartmentalize");
        // Create bootstrap change set
        let bootstrap_change_set_pk = dag.create_change_set(
            "bootstrap",
            "main",
            dag.get_head_for_change_set_name("main").unwrap(),
        );
        // Create 2 schemas
        let og_schema_a_pk = dag
            .create_schema(bootstrap_change_set_pk, "OG Schema A")
            .unwrap();
        let og_schema_b_pk = dag
            .create_schema(bootstrap_change_set_pk, "OG Schema B")
            .unwrap();
        println!("{:?}", petgraph::dot::Dot::with_config(&dag.graph, &[petgraph::dot::Config::EdgeNoLabel]));
        // Merge bootstrap change set
        dag.merge_change_set(bootstrap_change_set_pk).unwrap();
        println!("{:?}", petgraph::dot::Dot::with_config(&dag.graph, &[petgraph::dot::Config::EdgeNoLabel]));
        // Create change set
        let silo_change_set_pk = dag.create_change_set(
            "silo",
            "main",
            dag.get_head_for_change_set_name("main").unwrap(),
        );
        println!("{:?}", petgraph::dot::Dot::with_config(&dag.graph, &[petgraph::dot::Config::EdgeNoLabel]));
        // Modify schema A in the change set
        let schema_pk = dag
            .modify_schema(silo_change_set_pk, og_schema_a_pk, |s| {
                s.name = "Silo modified Schema A".to_string();
                Ok(())
            })
            .unwrap();
        // Make sure modified schema isn't in "main"
        println!("{:?}", petgraph::dot::Dot::with_config(&dag.graph, &[petgraph::dot::Config::EdgeNoLabel]));
        dbg!(dag.get_workspace_schemas(dag.get_change_set_by_name("main").unwrap().pk()).unwrap());
        dbg!(dag.get_workspace_schemas(dag.get_change_set_by_name("silo").unwrap().pk()).unwrap());
        assert!(!dag
            .is_node_index_in_change_set(
                dag.get_change_set_by_name("main").unwrap().pk(),
                dag.get_schema_node_index(schema_pk).unwrap(),
            )
            .unwrap());
        // Make sure modified schema is in the change set
        assert!(dag
            .is_node_index_in_change_set(
                silo_change_set_pk,
                dag.get_schema_node_index(schema_pk).unwrap()
            )
            .unwrap());
        // Make sure the unmodified schema is in "main"
        assert!(dag
            .is_node_index_in_change_set(
                dag.get_change_set_by_name("main").unwrap().pk(),
                dag.get_schema_node_index(og_schema_b_pk).unwrap(),
            )
            .unwrap());
        // Make sure the unmodified schema is in the change set
        assert!(dag
            .is_node_index_in_change_set(
                silo_change_set_pk,
                dag.get_schema_node_index(og_schema_b_pk).unwrap(),
            )
            .unwrap());
        // Make sure the OG schema A is in "main"
        assert!(dag
            .is_node_index_in_change_set(
                dag.get_change_set_by_name("main").unwrap().pk(),
                dag.get_schema_node_index(og_schema_a_pk).unwrap(),
            )
            .unwrap());
        // Make sure the OG schema A is not in the change set
        assert!(!dag
            .is_node_index_in_change_set(
                silo_change_set_pk,
                dag.get_schema_node_index(og_schema_a_pk).unwrap()
            )
            .unwrap());

        // Merge the change set
        dag.merge_change_set(silo_change_set_pk).unwrap();

        // Make sure modified schema is in "main"
        assert!(dag
            .is_node_index_in_change_set(
                dag.get_change_set_by_name("main").unwrap().pk(),
                dag.get_schema_node_index(schema_pk).unwrap(),
            )
            .unwrap());
        // Make sure modified schema is in the change set
        assert!(dag
            .is_node_index_in_change_set(
                silo_change_set_pk,
                dag.get_schema_node_index(schema_pk).unwrap()
            )
            .unwrap());
        // Make sure the unmodified schema is in "main"
        assert!(dag
            .is_node_index_in_change_set(
                dag.get_change_set_by_name("main").unwrap().pk(),
                dag.get_schema_node_index(og_schema_b_pk).unwrap(),
            )
            .unwrap());
        // Make sure the unmodified schema is in the change set
        assert!(dag
            .is_node_index_in_change_set(
                silo_change_set_pk,
                dag.get_schema_node_index(og_schema_b_pk).unwrap(),
            )
            .unwrap());
        // Make sure the OG schema A is not in "main"
        assert!(!dag
            .is_node_index_in_change_set(
                dag.get_change_set_by_name("main").unwrap().pk(),
                dag.get_schema_node_index(og_schema_a_pk).unwrap(),
            )
            .unwrap());
        // Make sure the OG schema A is not in the change set
        assert!(!dag
            .is_node_index_in_change_set(
                silo_change_set_pk,
                dag.get_schema_node_index(og_schema_a_pk).unwrap()
            )
            .unwrap());
    }

    #[test]
    fn rebase_change_set_with_conflicts() {
        // Create DAG
        let mut dag = SiDag::new("disconsonant");
        // Create bootstrap change set
        let bootstrap_change_set_pk = dag.create_change_set(
            "bootstrap",
            "main",
            dag.get_head_for_change_set_name("main").unwrap(),
        );
        // Create 4 schemas
        // Will be updated only on main
        let og_schema_a_pk = dag
            .create_schema(bootstrap_change_set_pk, "OG Schema A")
            .unwrap();
        // Will be updated on both main, and new
        let og_schema_b_pk = dag
            .create_schema(bootstrap_change_set_pk, "OG Schema B")
            .unwrap();
        // Will be updated only on new
        let og_schema_c_pk = dag
            .create_schema(bootstrap_change_set_pk, "OG Schema C")
            .unwrap();
        // Will not be modified
        let og_schema_d_pk = dag
            .create_schema(bootstrap_change_set_pk, "OG Schema D")
            .unwrap();
        dag.merge_change_set(bootstrap_change_set_pk).unwrap();
        println!("Schemas in main");
        for schema in dag
            .get_workspace_schemas(dag.get_change_set_by_name("main").unwrap().pk())
            .unwrap()
        {
            dbg!(schema.id, schema.pk, &schema.name, &schema.content_hash);
        }
        // Create change set "new"
        let new_change_set_pk = dag.create_change_set(
            "new",
            "main",
            dag.get_head_for_change_set_name("main").unwrap(),
        );

        // Modify schemas on "main"
        let main_modification_change_set_pk = dag.create_change_set(
            "main modification",
            "main",
            dag.get_head_for_change_set_name("main").unwrap(),
        );
        // Modify Schema A
        let main_modified_schema_a_pk = dag
            .modify_schema(main_modification_change_set_pk, og_schema_a_pk, |s| {
                s.name = "Schema A modified on main".to_string();
                Ok(())
            })
            .unwrap();
        // Modify Schema B in "main"
        let main_modified_schema_b_pk = dag
            .modify_schema(main_modification_change_set_pk, og_schema_b_pk, |s| {
                s.name = "Schema B modified on main".to_string();
                Ok(())
            })
            .unwrap();
        // Merge changes to "main"
        dag.merge_change_set(main_modification_change_set_pk)
            .unwrap();
        println!("Schemas in main");
        for schema in dag
            .get_workspace_schemas(dag.get_change_set_by_name("main").unwrap().pk())
            .unwrap()
        {
            dbg!(schema.id, schema.pk, &schema.name, &schema.content_hash);
        }

        // Modify Schema B in "new"
        let new_modified_schema_b_pk = dag
            .modify_schema(new_change_set_pk, og_schema_b_pk, |s| {
                s.name = "Schema B modified on new".to_string();
                Ok(())
            })
            .unwrap();
        // Modify Schema C in "new"
        let new_modified_schema_c_pk = dag
            .modify_schema(new_change_set_pk, og_schema_c_pk, |s| {
                s.name = "Schema C modified on new".to_string();
                Ok(())
            })
            .unwrap();
        // Schema D is untouched
        println!("Schemas in new");
        for schema in dag
            .get_workspace_schemas(dag.get_change_set_by_name("new").unwrap().pk())
            .unwrap()
        {
            dbg!(schema.id, schema.pk, &schema.name, &schema.content_hash);
        }
        // Rebase "new"
        let conflicts = dbg!(dag.rebase_change_set(new_change_set_pk).unwrap());

        dbg!(
            og_schema_a_pk,
            og_schema_b_pk,
            og_schema_c_pk,
            og_schema_d_pk,
            main_modified_schema_a_pk,
            main_modified_schema_b_pk,
            new_modified_schema_b_pk,
            new_modified_schema_c_pk,
            new_change_set_pk
        );
        // Only Schema B conflicts
        panic!();
    }
}
