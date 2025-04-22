use petgraph::prelude::*;
use std::collections::{HashMap, hash_map::Entry};

#[derive(Debug, Clone)]
pub struct DependencyGraph<T: Copy + std::cmp::Eq + std::cmp::PartialEq + std::hash::Hash> {
    graph: StableDiGraph<T, ()>,
    id_to_index_map: HashMap<T, NodeIndex>,
}

impl<T: Copy + std::cmp::Eq + std::cmp::PartialEq + std::hash::Hash> Default
    for DependencyGraph<T>
{
    fn default() -> Self {
        Self {
            graph: Default::default(),
            id_to_index_map: Default::default(),
        }
    }
}

impl<T: Copy + std::cmp::Eq + std::cmp::PartialEq + std::hash::Hash> DependencyGraph<T> {
    pub fn new() -> Self {
        Self {
            id_to_index_map: HashMap::new(),
            graph: StableGraph::new(),
        }
    }

    pub fn add_id(&mut self, new_id: T) -> NodeIndex {
        match self.id_to_index_map.entry(new_id) {
            Entry::Vacant(entry) => {
                let node_idx = self.graph.add_node(new_id);
                entry.insert(node_idx);

                node_idx
            }
            Entry::Occupied(entry) => *entry.get(),
        }
    }

    pub fn id_depends_on(&mut self, id: T, depends_on_id: T) {
        let value_idx = self.add_id(id);
        let depends_on_idx = self.add_id(depends_on_id);

        self.graph.update_edge(value_idx, depends_on_idx, ());
    }

    pub fn contains_id(&self, id: T) -> bool {
        self.id_to_index_map.contains_key(&id)
    }

    pub fn direct_dependencies_of(&self, id: T) -> Vec<T> {
        match self.id_to_index_map.get(&id) {
            None => vec![],
            Some(value_idx) => self
                .graph
                .edges_directed(*value_idx, Outgoing)
                .filter_map(|edge_ref| self.graph.node_weight(edge_ref.target()).copied())
                .collect(),
        }
    }

    pub fn direct_reverse_dependencies_of(&self, id: T) -> Vec<T> {
        match self.id_to_index_map.get(&id) {
            None => vec![],
            Some(value_idx) => self
                .graph
                .edges_directed(*value_idx, Incoming)
                .filter_map(|edge_ref| self.graph.node_weight(edge_ref.source()).copied())
                .collect(),
        }
    }

    pub fn remove_id(&mut self, id: T) {
        if let Some(node_idx) = self.id_to_index_map.remove(&id) {
            self.graph.remove_node(node_idx);
        }
    }

    pub fn cycle_on_self(&mut self, id: T) {
        if let Some(node_idx) = self.id_to_index_map.get(&id) {
            self.graph.add_edge(*node_idx, *node_idx, ());
        }
    }

    pub fn independent_ids(&self) -> Vec<T> {
        self.graph
            .externals(Outgoing)
            .filter_map(|node_idx| self.graph.node_weight(node_idx).copied())
            .collect()
    }

    pub fn remaining_ids(&self) -> Vec<T> {
        self.graph.node_weights().copied().collect()
    }

    pub fn into_graph(self) -> StableDiGraph<T, ()> {
        self.graph
    }

    pub fn graph(&self) -> &StableDiGraph<T, ()> {
        &self.graph
    }

    pub fn id_to_index_map(&self) -> &HashMap<T, NodeIndex> {
        &self.id_to_index_map
    }

    pub fn all_ids(&self) -> Vec<T> {
        self.graph.node_weights().copied().collect()
    }
}
