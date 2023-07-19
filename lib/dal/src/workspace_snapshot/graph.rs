use petgraph::algo;
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::workspace_snapshot::{
    edge_weight::EdgeWeight,
    node_weight::{NodeWeight, NodeWeightKind},
    WorkspaceSnapshotError, WorkspaceSnapshotResult,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraph {
    pub root_index: NodeIndex,
    pub graph: StableDiGraph<NodeWeight, EdgeWeight>,
}
impl Default for WorkspaceSnapshotGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceSnapshotGraph {
    pub fn new() -> Self {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(1, 0);
        let root_index = graph.add_node(NodeWeight::new(NodeWeightKind::Root));
        Self { root_index, graph }
    }

    pub fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    pub fn cleanup(&mut self) {
        self.graph.retain_nodes(|frozen_graph, current_node| {
            // We cannot use "has_path_to_root" because we need the frozen graph.
            algo::has_path_connecting(&*frozen_graph, self.root_index, current_node, None)
        });
    }

    fn dot(&self) {
        // NOTE(nick): copy the output and execute this on macOS. It will create two files in the
        // process and open a new tab in your browser.
        // ```
        // pbpaste > foo.txt && dot foo.txt -Tsvg -o foo.svg && open foo.svg
        // ```
        println!(
            "{:?}",
            petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
        );
    }

    fn add_node(&mut self, node: NodeWeight) -> NodeIndex {
        self.graph.add_node(node)
    }

    fn add_edge(
        &mut self,
        edge: EdgeWeight,
        parent_node_index: NodeIndex,
        node_index: NodeIndex,
    ) -> EdgeIndex {
        self.graph.add_edge(parent_node_index, node_index, edge)
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
                        let new_node_index =
                            self.add_node(*self.get_node_weight(old_node_index)?);
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
                        edge_weight,
                        new_node_index,
                        *old_to_new_node_indices
                            .get(&destination_node_index)
                            .unwrap_or(&destination_node_index),
                    );
                }
            }
        }

        // Use the new version of the old root node as our root node.
        if let Some(new_root_node_index) = old_to_new_node_indices.get(&self.root_index) {
            self.root_index = *new_root_node_index;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ComponentId, FuncId, PropId, SchemaId, SchemaVariantId};

    #[test]
    fn new() {
        let graph = WorkspaceSnapshotGraph::new();
        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn add_nodes_and_edges() {
        let mut graph = WorkspaceSnapshotGraph::new();

        let schema_index = graph.add_node(NodeWeight::new(NodeWeightKind::Schema(
            SchemaId::generate(),
        )));
        let schema_variant_index = graph.add_node(NodeWeight::new(NodeWeightKind::SchemaVariant(
            SchemaVariantId::generate(),
        )));
        let component_index = graph.add_node(NodeWeight::new(NodeWeightKind::Component(
            ComponentId::generate(),
        )));

        graph.add_edge(EdgeWeight::default(), graph.root_index, component_index);
        graph.add_edge(EdgeWeight::default(), graph.root_index, schema_index);
        graph.add_edge(EdgeWeight::default(), schema_index, schema_variant_index);
        graph.add_edge(EdgeWeight::default(), component_index, schema_variant_index);

        let func_index = graph.add_node(NodeWeight::new(NodeWeightKind::Func(FuncId::generate())));
        let prop_index = graph.add_node(NodeWeight::new(NodeWeightKind::Prop(PropId::generate())));

        graph.add_edge(EdgeWeight::default(), graph.root_index, func_index);
        graph.add_edge(EdgeWeight::default(), schema_variant_index, prop_index);
        graph.add_edge(EdgeWeight::default(), prop_index, func_index);

        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn cyclic_failure() {
        let mut graph = WorkspaceSnapshotGraph::new();

        let schema_index = graph.add_node(NodeWeight::new(NodeWeightKind::Schema(
            SchemaId::generate(),
        )));
        let schema_variant_index = graph.add_node(NodeWeight::new(NodeWeightKind::SchemaVariant(
            SchemaVariantId::generate(),
        )));
        let component_index = graph.add_node(NodeWeight::new(NodeWeightKind::Component(
            ComponentId::generate(),
        )));

        graph.add_edge(EdgeWeight::default(), graph.root_index, component_index);
        graph.add_edge(EdgeWeight::default(), graph.root_index, schema_index);
        graph.add_edge(EdgeWeight::default(), schema_index, schema_variant_index);
        graph.add_edge(EdgeWeight::default(), component_index, schema_variant_index);

        // This should cause a cycle.
        graph.add_edge(EdgeWeight::default(), schema_variant_index, component_index);

        assert!(!graph.is_acyclic_directed());
    }

    #[test]
    fn replace_references() {
        let mut graph = WorkspaceSnapshotGraph::new();

        let schema_index = graph.add_node(NodeWeight::new(NodeWeightKind::Schema(
            SchemaId::generate(),
        )));
        let schema_variant_index = graph.add_node(NodeWeight::new(NodeWeightKind::SchemaVariant(
            SchemaVariantId::generate(),
        )));
        let component_index = graph.add_node(NodeWeight::new(NodeWeightKind::Component(
            ComponentId::generate(),
        )));

        graph.add_edge(EdgeWeight::default(), graph.root_index, component_index);
        graph.add_edge(EdgeWeight::default(), graph.root_index, schema_index);
        graph.add_edge(EdgeWeight::default(), schema_index, schema_variant_index);
        graph.add_edge(EdgeWeight::default(), component_index, schema_variant_index);

        let new_component_index = graph.add_node(NodeWeight::new(NodeWeightKind::Component(
            ComponentId::generate(),
        )));
        graph
            .replace_references(component_index, new_component_index)
            .expect("could not replace references");

        // TODO(nick,jacob): do something here
    }

    #[test]
    fn replace_references_and_cleanup() {
        let mut graph = WorkspaceSnapshotGraph::new();

        let schema_index = graph.add_node(NodeWeight::new(NodeWeightKind::Schema(
            SchemaId::generate(),
        )));
        let schema_variant_index = graph.add_node(NodeWeight::new(NodeWeightKind::SchemaVariant(
            SchemaVariantId::generate(),
        )));
        let component_index = graph.add_node(NodeWeight::new(NodeWeightKind::Component(
            ComponentId::generate(),
        )));

        graph.add_edge(EdgeWeight::default(), graph.root_index, component_index);
        graph.add_edge(EdgeWeight::default(), graph.root_index, schema_index);
        graph.add_edge(EdgeWeight::default(), schema_index, schema_variant_index);
        graph.add_edge(EdgeWeight::default(), component_index, schema_variant_index);

        let new_component_index = graph.add_node(NodeWeight::new(NodeWeightKind::Component(
            ComponentId::generate(),
        )));
        graph
            .replace_references(component_index, new_component_index)
            .expect("could not replace references");

        graph.cleanup();

        // TODO(nick,jacob): do something here
    }
}
