use std::collections::HashMap;

use petgraph::{prelude::*, visit::Walker};
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
    pub graph: Graph<SiNode, SiEdge>,
    pub heads: HashMap<ChangeSetPk, WorkspacePk>,
    pub change_sets: HashMap<ChangeSetPk, ChangeSet>,
    pub workspaces: HashMap<WorkspacePk, IndexAndEntry<Workspace>>,
    pub schemas: HashMap<SchemaPk, IndexAndEntry<Schema>>,
    pub vector_clocks: HashMap<Ulid, VectorClock>,
}

impl SiDag {
    pub fn new(workspace_name: impl Into<String>) -> SiDag {
        let workspace_name = workspace_name.into();
        let mut dag = SiDag {
            heads: HashMap::new(),
            change_sets: HashMap::new(),
            graph: Graph::new(),
            workspaces: HashMap::new(),
            schemas: HashMap::new(),
            vector_clocks: HashMap::new(),
        };
        let workspace_pk = dag.create_workspace(workspace_name);
        let workspace = dag.get_workspace(workspace_pk).unwrap();
        let change_set_pk = dag.create_change_set("main", workspace.id(), workspace.pk());
        dag.vector_clocks
            .insert(workspace_pk, VectorClock::new(workspace_pk, change_set_pk));
        dag
    }

    pub fn merge_change_set(&mut self, change_set_pk: ChangeSetPk) -> DagResult<()> {
        let to_merge_change_set = self.get_change_set_by_pk(change_set_pk)?;
        let to_merge_workspace_pk = self.get_head_for_change_set_pk(change_set_pk)?;
        let target_change_set =
            self.get_change_set_by_name(&to_merge_change_set.target_change_set_name)?;
        let target_workspace_pk = self.get_head_for_change_set_pk(target_change_set.pk())?;

        if to_merge_change_set.base_workspace_pk != target_workspace_pk {
            return Err(DagError::MustRebase);
        }

        self.heads
            .insert(target_change_set.pk(), to_merge_workspace_pk);
        Ok(())
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
    pub fn rebase_change_set(&mut self, change_set_pk: ChangeSetPk) -> DagResult<()> {
        let change_set = self.get_change_set_by_pk(change_set_pk)?;

        // Get the merge base workspace
        let target_workspace_pk =
            self.get_head_for_change_set_name(&change_set.target_change_set_name)?;
        let target_workspace_idx = self.get_workspace_node_index(target_workspace_pk)?;

        let mut bfs = Bfs::new(&self.graph, target_workspace_idx);
        while let Some(node_index) = bfs.next(&self.graph) {
            match self
                .graph
                .node_weight(node_index)
                .ok_or(DagError::MissingNodeWeight)? {
                    SiNode { kind: SiNodeKind::Workspace(workspace_pk) } => {

                    },
                    _ => {}
                }
        }
        todo!()
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
        Ok(self
            .change_sets
            .get(&change_set_pk)
            .ok_or(DagError::ChangeSetNotFound(change_set_pk))?)
    }

    pub fn get_change_set_by_name(&self, name: impl Into<String>) -> DagResult<&ChangeSet> {
        let name = name.into();
        Ok(self
            .change_sets
            .values()
            .find(|cs| cs.name == name)
            .ok_or(DagError::ChangeSetNameNotFound(name))?)
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

        let mut new_workspace = base_object.clone();
        new_workspace.pk = WorkspacePk::new();
        let new_workspace_pk = new_workspace.pk();
        let new_index = self
            .graph
            .add_node(SiNode::new(SiNodeKind::Workspace(new_workspace.pk())));

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

        self.vector_clocks
            .entry(workspace_pk)
            .and_modify(|vc| vc.inc(change_set_pk))
            .or_insert(VectorClock::new(workspace_pk, change_set_pk));

        self.heads.insert(change_set_pk, new_workspace_pk);

        Ok(new_workspace_pk)
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
        Ok(self
            .workspaces
            .get(&workspace_pk)
            .map(|e| e.entry())
            .ok_or(DagError::WorkspaceNotFound(workspace_pk))?)
    }

    pub fn get_workspace_node_index(&self, workspace_pk: WorkspacePk) -> DagResult<NodeIndex> {
        Ok(self
            .workspaces
            .get(&workspace_pk)
            .map(|e| e.node_index())
            .ok_or(DagError::WorkspaceNotFound(workspace_pk))?)
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
            .or_insert(VectorClock::new(schema_pk, change_set_pk));

        Ok(schema_pk)
    }

    pub fn get_schema_by_pk(&self, pk: SchemaPk) -> DagResult<&Schema> {
        Ok(self
            .schemas
            .get(&pk)
            .map(|e| e.entry())
            .ok_or(DagError::SchemaNotFound(pk))?)
    }

    pub fn get_schema_node_index(&self, pk: SchemaPk) -> DagResult<NodeIndex> {
        Ok(self
            .schemas
            .get(&pk)
            .map(|e| e.node_index())
            .ok_or(DagError::SchemaNotFound(pk))?)
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

        let mut workspaces_to_update = Vec::new();
        // Trigger recalulating the other content hashes
        for node_idx in self.graph.neighbors_directed(index, Direction::Incoming) {
            match self.graph.node_weight(node_idx) {
                Some(SiNode {
                    kind: SiNodeKind::Workspace(workspace_pk),
                }) => {
                    workspaces_to_update.push(*workspace_pk);
                }
                _ => continue,
            }
        }
        for workspace_pk in workspaces_to_update {
            self.update_workspace_content_hash(workspace_pk)?;
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
}

#[derive(Debug, Clone)]
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
        drop(killswitch_workspace_pk);

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
    fn not_rebased_merge_fails() {
        // Create a new dag
        let mut dag = SiDag::new("poop");

        // Get the head workspace
        let head_workspace_pk = dag.get_head_for_change_set_name("main").unwrap();

        // Create a pair of change sets
        let killswitch_change_set_pk =
            dag.create_change_set("killswitch", "main", head_workspace_pk);
        let slayer_change_set_pk = dag.create_change_set("slayer", "main", head_workspace_pk);

        // Modify the workspace in the killswitch change set
        let _modified_workspace_pk = dag
            .modify_workspace(killswitch_change_set_pk, |w| {
                w.name = "back to the future".to_string();
                Ok(())
            })
            .unwrap();

        dag.merge_change_set(killswitch_change_set_pk).unwrap();

        // Should fail
        assert_eq!(
            dag.merge_change_set(slayer_change_set_pk).unwrap_err(),
            DagError::MustRebase
        );
    }

    #[test]
    fn rebase() {

    }

    //#[test]
    //fn change_set_create() {
    //    // Create a new workspace, update the dag
    //    let mut dag = SiDag::new("poop");
    //    let change_set_id = dag.create_change_set("floop");
    //}

    //#[test]
    //fn add_schema_to_workspace() {
    //    // Create the universe
    //    let mut dag = SiDag::new();

    //    // Create a new workspace; it is empty.
    //    let first_workspace_id = dag.create_workspace("Dr Who");

    //    // Create a new schema in our workspace.
    //    let schema_id = dag.create_schema(first_workspace_id, "The 1975").unwrap();

    //    // We now have two workspaces
    //    assert_eq!(dag.workspaces.len(), 2);

    //    //let schema = dag.get_schema(schema_id).unwrap();
    //    //assert_eq!(schema_id, schema.id());
    //    //let workspace_schemas = dag
    //    //    .get_workspace_schemas(workspace_id)
    //    //    .expect("cannot get workspace schemas");
    //    //assert_eq!(workspace_schemas.len(), 1);
    //    //assert_eq!(workspace_schemas[0].id(), schema.id());
    //    //assert_eq!(workspace_schemas[0].name, "The 1975");
    //}
}
