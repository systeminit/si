use petgraph::algo;
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::{
    change_set::ChangeSet,
    content_hash::ContentHash,
    edge_weight::EdgeWeight,
    node_weight::{NodeWeight, NodeWeightError, NodeWeightKind},
    WorkspaceSnapshotError, WorkspaceSnapshotResult,
};

#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
}

pub type WorkspaceSnapshotGraphResult<T> = Result<T, WorkspaceSnapshotGraphError>;

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraph {
    root_index: NodeIndex,
    graph: StableDiGraph<NodeWeight, EdgeWeight>,
}

impl std::fmt::Debug for WorkspaceSnapshotGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshotGraph")
            .field("root_index", &self.root_index)
            .field("graph", &self.graph)
            .finish()
    }
}

impl WorkspaceSnapshotGraph {
    pub fn new(change_set: &ChangeSet) -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(1, 0);
        let root_index = graph.add_node(NodeWeight::new(change_set, NodeWeightKind::Root)?);

        Ok(Self { root_index, graph })
    }

    fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    pub fn cleanup(&mut self) {
        self.graph.retain_nodes(|frozen_graph, current_node| {
            // We cannot use "has_path_to_root" because we need to use the Frozen<StableGraph<...>>.
            algo::has_path_connecting(&*frozen_graph, self.root_index, current_node, None)
        });
    }

    fn dot(&self) {
        // NOTE(nick): copy the output and execute this on macOS. It will create two files in the
        // process and open a new tab in your browser.
        // ```
        // pbpaste | dot -Tsvg -o foo.svg && open foo.svg
        // ```
        println!(
            "{:?}",
            petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
        );
    }

    fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.graph.add_node(node);
        self.update_merkle_tree_hash(new_node_index)?;

        Ok(new_node_index)
    }

    fn add_edge(
        &mut self,
        from_node_index: NodeIndex,
        edge: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        let new_edge_index = self.graph.add_edge(from_node_index, to_node_index, edge);
        if !self.is_acyclic_directed() {
            self.graph.remove_edge(new_edge_index);
            return Err(WorkspaceSnapshotError::CreateGraphCycle);
        }
        self.update_merkle_tree_hash(from_node_index)?;

        Ok(new_edge_index)
    }

    fn get_node_index_by_id(&mut self, id: Ulid) -> WorkspaceSnapshotResult<Option<NodeIndex>> {
        for node_index in self.graph.node_indices() {
            let node_weight = self.get_node_weight(node_index)?;
            if node_weight.id == id {
                return Ok(Some(node_index));
            }
        }

        Ok(None)
    }

    fn has_path_to_root(&self, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, self.root_index, node, None)
    }

    fn is_on_path_between(&self, start: NodeIndex, end: NodeIndex, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, start, node, None)
            && algo::has_path_connecting(&self.graph, node, end, None)
    }

    fn get_node_weight(&self, node_index: NodeIndex) -> WorkspaceSnapshotResult<&NodeWeight> {
        self.graph
            .node_weight(node_index)
            .ok_or(WorkspaceSnapshotError::NodeWeightNotFound)
    }

    fn replace_references(
        &mut self,
        change_set: &ChangeSet,
        original_node_index: NodeIndex,
        new_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        let mut old_to_new_node_indices: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        old_to_new_node_indices.insert(original_node_index, new_node_index);

        let mut dfspo = DfsPostOrder::new(&self.graph, self.root_index);
        while let Some(old_node_index) = dfspo.next(&self.graph) {
            // All nodes that exist between the root and the `original_node_index` are affected by the replace, and only
            // those nodes are affected, because the replacement affects their merkel tree hashes.
            if self.is_on_path_between(self.root_index, original_node_index, old_node_index) {
                // Copy the node if we have not seen it or grab it if we have. Only the first node in DFS post order
                // traversal should already exist since it was created before we entered `replace_references`, and
                // is the reason we're updating things in the first place.
                let new_node_index = match old_to_new_node_indices.get(&old_node_index) {
                    Some(found_new_node_index) => *found_new_node_index,
                    None => {
                        let new_node_index = self
                            .add_node(self.get_node_weight(old_node_index)?.modify(change_set)?)?;
                        old_to_new_node_indices.insert(old_node_index, new_node_index);
                        new_node_index
                    }
                };

                // Find all outgoing edges. From those outgoing edges and find their destinations.
                // If they do not have destinations, then there is no work to do (i.e. stale edge
                // reference, likely never going to happen).
                let mut edges_to_create: Vec<(EdgeWeight, NodeIndex)> = Vec::new();
                for edge_reference in self.graph.edges_directed(old_node_index, Outgoing) {
                    let edge_weight = edge_reference.weight();
                    if let Some((_, destination_node_index)) =
                        self.graph.edge_endpoints(edge_reference.id())
                    {
                        edges_to_create.push((*edge_weight, destination_node_index));
                    }
                }

                // Make copies of these edges where the source is the new node index and the
                // destination is one of the following...
                // - If an entry exists in `old_to_new_node_indicies` for the destination node index,
                //   use the value of the entry (the destination was affected by the replacement,
                //   and needs to use the new node index to reflect this).
                // - There is no entry in `old_to_new_node_indicies`; use the same destination node
                //   index as the old edge (the destination was *NOT* affected by the replacemnt,
                //   and does not have any new information to reflect).
                for (edge_weight, destination_node_index) in edges_to_create {
                    self.add_edge(
                        new_node_index,
                        edge_weight,
                        *old_to_new_node_indices
                            .get(&destination_node_index)
                            .unwrap_or(&destination_node_index),
                    )?;
                }

                self.update_merkle_tree_hash(new_node_index)?;

                // Use the new version of the old root node as our root node.
                if let Some(new_root_node_index) = old_to_new_node_indices.get(&self.root_index) {
                    self.root_index = *new_root_node_index;
                }
            }
        }

        Ok(())
    }

    fn update_merkle_tree_hash(
        &mut self,
        node_index_to_update: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        let mut hasher = ContentHash::hasher();
        hasher.update(
            self.get_node_weight(node_index_to_update)?
                .content_hash()
                .to_string()
                .as_bytes(),
        );

        // Need to make sure the neighbors are added to the hash in a stable order to ensure the
        // merkle tree hash is identical for identical trees.
        let mut ordered_neighbors = Vec::new();
        for neighbor_node in self
            .graph
            .neighbors_directed(node_index_to_update, Outgoing)
        {
            ordered_neighbors.push(neighbor_node);
        }
        ordered_neighbors.sort();

        for neighbor_node in ordered_neighbors {
            hasher.update(
                self.graph
                    .node_weight(neighbor_node)
                    .ok_or(WorkspaceSnapshotError::NodeWeightNotFound)?
                    .merkle_tree_hash()
                    .to_string()
                    .as_bytes(),
            );
        }

        let new_node_weight = self
            .graph
            .node_weight_mut(node_index_to_update)
            .ok_or(WorkspaceSnapshotError::NodeWeightNotFound)?;
        new_node_weight.set_merkle_tree_hash(hasher.finalize());

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        workspace_snapshot::content_hash::ContentHash, ComponentId, FuncId, PropId, SchemaId,
        SchemaVariantId,
    };

    #[test]
    fn new() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let graph = WorkspaceSnapshotGraph::new(&change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn add_nodes_and_edges() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(graph.root_index, EdgeWeight::default(), component_index)
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(graph.root_index, EdgeWeight::default(), schema_index)
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(schema_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(component_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add component -> schema variant edge");

        let func_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Func(ContentHash::new(
                        FuncId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add func");
        let prop_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Prop(ContentHash::new(
                        PropId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add prop");

        graph
            .add_edge(graph.root_index, EdgeWeight::default(), func_index)
            .expect("Unable to add root -> func edge");
        graph
            .add_edge(schema_variant_index, EdgeWeight::default(), prop_index)
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(prop_index, EdgeWeight::default(), func_index)
            .expect("Unable to add prop -> func edge");

        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn cyclic_failure() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(graph.root_index, EdgeWeight::default(), component_index)
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(graph.root_index, EdgeWeight::default(), schema_index)
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(schema_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(component_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add component -> schema variant edge");

        // This should cause a cycle.
        graph
            .add_edge(schema_variant_index, EdgeWeight::default(), component_index)
            .expect_err("Created a cycle");
    }

    #[test]
    fn replace_references() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(graph.root_index, EdgeWeight::default(), component_index)
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(graph.root_index, EdgeWeight::default(), schema_index)
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(schema_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(component_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add component -> schema variant edge");

        // TODO: This is meant to simulate "modifying" the existing component, instead of swapping in a completely independent component.
        let new_component_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add new component");
        graph
            .replace_references(change_set, component_index, new_component_index)
            .expect("could not replace references");

        // TODO(nick,jacob): do something here
    }

    #[test]
    fn replace_references_and_cleanup() {
        let change_set = ChangeSet::new().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(graph.root_index, EdgeWeight::default(), component_index)
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(graph.root_index, EdgeWeight::default(), schema_index)
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(schema_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(component_index, EdgeWeight::default(), schema_variant_index)
            .expect("Unable to add component -> schema variant edge");

        graph.dot();

        // TODO: This is meant to simulate "modifying" the existing component, instead of swapping in a completely independent component.
        let new_component_index = graph
            .add_node(
                NodeWeight::new(
                    change_set,
                    NodeWeightKind::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add new component");
        graph
            .replace_references(change_set, component_index, new_component_index)
            .expect("could not replace references");

        graph.dot();

        graph.cleanup();

        graph.dot();

        panic!();

        // TODO(nick,jacob): do something here
    }
}
