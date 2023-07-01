use petgraph::prelude::*;

#[derive(Debug, Clone)]
pub struct IndexAndEntry<T> {
    pub node_index: NodeIndex,
    pub entry: T,
}

impl<T> IndexAndEntry<T> {
    pub fn new(node_index: NodeIndex, entry: T) -> IndexAndEntry<T> {
        IndexAndEntry { node_index, entry }
    }

    pub fn node_index(&self) -> NodeIndex {
        self.node_index
    }

    pub fn entry(&self) -> &T {
        &self.entry
    }
}

