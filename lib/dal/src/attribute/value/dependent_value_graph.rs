use petgraph::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use super::AttributeValueId;

#[derive(Debug, Clone)]
pub struct DependentValueGraph {
    graph: StableDiGraph<AttributeValueId, ()>,
    id_to_index_map: HashMap<AttributeValueId, NodeIndex>,
}

impl Default for DependentValueGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependentValueGraph {
    pub fn new() -> Self {
        Self {
            id_to_index_map: HashMap::new(),
            graph: StableGraph::new(),
        }
    }

    pub fn add_value(&mut self, value_id: AttributeValueId) -> NodeIndex {
        match self.id_to_index_map.entry(value_id) {
            Entry::Vacant(entry) => {
                let node_idx = self.graph.add_node(value_id);
                entry.insert(node_idx);

                node_idx
            }
            Entry::Occupied(entry) => *entry.get(),
        }
    }

    pub fn value_depends_on(
        &mut self,
        value_id: AttributeValueId,
        depends_on_id: AttributeValueId,
    ) {
        let value_idx = self.add_value(value_id);
        let depends_on_idx = self.add_value(depends_on_id);

        self.graph.add_edge(value_idx, depends_on_idx, ());
    }

    pub fn contains_value(&self, value_id: AttributeValueId) -> bool {
        self.id_to_index_map.get(&value_id).is_some()
    }

    pub fn direct_dependencies_of(&self, value_id: AttributeValueId) -> Vec<AttributeValueId> {
        match self.id_to_index_map.get(&value_id) {
            None => vec![],
            Some(value_idx) => self
                .graph
                .edges_directed(*value_idx, Outgoing)
                .filter_map(|edge_ref| self.graph.node_weight(edge_ref.target()).copied())
                .collect(),
        }
    }

    pub fn remove_value(&mut self, value_id: AttributeValueId) {
        if let Some(node_idx) = self.id_to_index_map.remove(&value_id) {
            self.graph.remove_node(node_idx);
        }
    }

    pub fn cycle_on_self(&mut self, value_id: AttributeValueId) {
        if let Some(node_idx) = self.id_to_index_map.get(&value_id) {
            self.graph.add_edge(*node_idx, *node_idx, ());
        }
    }

    pub fn independent_values(&self) -> Vec<AttributeValueId> {
        self.graph
            .externals(Outgoing)
            .filter_map(|node_idx| self.graph.node_weight(node_idx).copied())
            .collect()
    }

    pub fn into_graph(self) -> StableDiGraph<AttributeValueId, ()> {
        self.graph
    }
}
