use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_id::ulid::Ulid;
use std::collections::{HashMap, HashSet};
use std::io::Write;

use crate::{
    CustomEdgeWeight, CustomNodeWeight, EdgeKind, SplitGraphEdgeWeight, SplitGraphNodeId,
    SplitGraphNodeWeight, MAX_NODES,
};

pub type SubGraphNodeIndex = NodeIndex<u16>;
pub type SubGraphEdgeIndex = EdgeIndex<u16>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(super) graph: StableDiGraph<SplitGraphNodeWeight<N>, SplitGraphEdgeWeight<E, K>, u16>,
    pub(super) node_index_by_id: HashMap<SplitGraphNodeId, SubGraphNodeIndex>,
    pub(super) root_index: SubGraphNodeIndex,

    #[serde(skip)]
    pub(super) touched_nodes: HashSet<SubGraphNodeIndex>,
}

impl<N, E, K> Default for SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E, K> SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub fn new() -> Self {
        Self {
            graph: StableDiGraph::with_capacity(MAX_NODES, MAX_NODES * 2),
            node_index_by_id: HashMap::new(),
            root_index: NodeIndex::new(0),

            touched_nodes: HashSet::new(),
        }
    }

    pub fn cleanup(&mut self) {
        loop {
            let orphaned_node_indexes: Vec<SubGraphNodeIndex> = self
                .graph
                .externals(Incoming)
                .filter(|idx| *idx != self.root_index)
                .collect();

            if orphaned_node_indexes.is_empty() {
                break;
            }

            for node_index in orphaned_node_indexes {
                self.graph.remove_node(node_index);
            }
        }

        self.node_index_by_id
            .retain(|_id, index| self.graph.node_weight(*index).is_some());
    }

    pub(super) fn add_node(&mut self, node: SplitGraphNodeWeight<N>) -> SubGraphNodeIndex {
        let node_id = node.id();
        let node_index = self.graph.add_node(node);
        self.node_index_by_id.insert(node_id, node_index);
        self.touched_nodes.insert(node_index);

        node_index
    }

    pub(super) fn replace_node(&mut self, index: SubGraphNodeIndex, node: SplitGraphNodeWeight<N>) {
        if let Some(node_ref) = self.graph.node_weight_mut(index) {
            *node_ref = node;
        }
        self.touched_nodes.insert(index);
    }

    fn edge_exists(
        &self,
        from_index: SubGraphNodeIndex,
        edge_weight: &SplitGraphEdgeWeight<E, K>,
        to_index: SubGraphNodeIndex,
    ) -> bool {
        self.graph
            .edges_connecting(from_index, to_index)
            .any(|edge_ref| match edge_ref.weight() {
                SplitGraphEdgeWeight::Custom(custom_edge) => {
                    Some(custom_edge.kind()) == edge_weight.custom().map(|e| e.kind())
                }
                SplitGraphEdgeWeight::ExternalSource {
                    source_id,
                    edge_kind,
                    ..
                } => match &edge_weight {
                    SplitGraphEdgeWeight::ExternalSource {
                        source_id: new_source_id,
                        edge_kind: new_edge_kind,
                        ..
                    } => source_id == new_source_id && edge_kind == new_edge_kind,
                    _ => false,
                },
                SplitGraphEdgeWeight::Ordering => {
                    matches!(edge_weight, SplitGraphEdgeWeight::Ordering)
                }
                SplitGraphEdgeWeight::Ordinal => {
                    matches!(edge_weight, SplitGraphEdgeWeight::Ordinal)
                }
            })
    }

    pub(super) fn add_edge(
        &mut self,
        from_index: SubGraphNodeIndex,
        edge_weight: SplitGraphEdgeWeight<E, K>,
        to_index: SubGraphNodeIndex,
    ) {
        let exists = self.edge_exists(from_index, &edge_weight, to_index);
        let is_ordered_container = self
            .graph
            .node_weight(from_index)
            .and_then(|weight| weight.custom().map(|c| c.ordered()))
            .is_some_and(|ordered| ordered);

        if is_ordered_container {
            let target_id = self.graph.node_weight(to_index).map(|n| n.id()).unwrap();
            let ordering_node_index = match self
                .graph
                .edges_directed(from_index, Outgoing)
                .find(|edge_ref| matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordering))
                .map(|edge_ref| edge_ref.target())
            {
                Some(target) => target,
                None => self.graph.add_node(SplitGraphNodeWeight::Ordering {
                    id: Ulid::new(),
                    order: vec![],
                    merkle_tree_hash: MerkleTreeHash::nil(),
                }),
            };

            if let Some(SplitGraphNodeWeight::Ordering { order, .. }) =
                self.graph.node_weight_mut(ordering_node_index)
            {
                if !order.contains(&target_id) {
                    order.push(target_id);
                }
                if !self.edge_exists(
                    ordering_node_index,
                    &SplitGraphEdgeWeight::Ordinal,
                    to_index,
                ) {
                    self.graph.add_edge(
                        ordering_node_index,
                        to_index,
                        SplitGraphEdgeWeight::Ordinal,
                    );
                }
            }
        }

        if !exists {
            self.graph.add_edge(from_index, to_index, edge_weight);
            self.touch_node(from_index);
        }
    }

    fn touch_node(&mut self, node_index: SubGraphNodeIndex) {
        self.touched_nodes.insert(node_index);
    }

    pub(super) fn remove_edge(&mut self, edge_index: EdgeIndex<u16>) {
        if let Some((from_index, to_index)) = self.graph.edge_endpoints(edge_index) {
            self.touch_node(from_index);

            let is_ordered_container = self
                .graph
                .node_weight(from_index)
                .and_then(|weight| weight.custom().map(|c| c.ordered()))
                .is_some_and(|ordered| ordered);

            if is_ordered_container {
                let target_id = self.graph.node_weight(to_index).map(|n| n.id()).unwrap();
                if let Some(ordering_node_index) = self
                    .graph
                    .edges_directed(from_index, Outgoing)
                    .find(|edge_ref| matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordering))
                    .map(|edge_ref| edge_ref.target())
                {
                    self.touch_node(ordering_node_index);
                    if let Some(ordinal_edge_index) = self
                        .graph
                        .edges_directed(ordering_node_index, Outgoing)
                        .find(|edge_ref| {
                            matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordinal)
                                && self.graph.node_weight(edge_ref.target()).map(|n| n.id())
                                    == Some(target_id)
                        })
                        .map(|edge_ref| edge_ref.id())
                    {
                        self.graph.remove_edge(ordinal_edge_index);

                        if let Some(SplitGraphNodeWeight::Ordering { order, .. }) =
                            self.graph.node_weight_mut(ordering_node_index)
                        {
                            order.retain(|id| *id != target_id);
                        }
                    }
                }
            }

            self.graph.remove_edge(edge_index);
        }
    }

    pub fn tiny_dot_to_file(&self, name: &str) {
        let dot = petgraph::dot::Dot::with_attr_getters(
            &self.graph,
            &[
                petgraph::dot::Config::NodeNoLabel,
                petgraph::dot::Config::EdgeNoLabel,
            ],
            &|_, edge_ref| {
                let (label, color) = match edge_ref.weight() {
                    SplitGraphEdgeWeight::Custom(_) => ("".into(), "black"),
                    SplitGraphEdgeWeight::ExternalSource {
                        source_id,
                        subgraph,
                        ..
                    } => (
                        format!("external source: {source_id}\nsubgraph: {}", subgraph + 1),
                        "red",
                    ),
                    SplitGraphEdgeWeight::Ordering => ("ordering".into(), "green"),
                    SplitGraphEdgeWeight::Ordinal => ("ordinal".into(), "green"),
                };

                format!("label = \"{label}\"\ncolor = {color}")
            },
            &|_, (_, node_weight)| {
                let (label, color) = match node_weight {
                    SplitGraphNodeWeight::Custom(n) => {
                        let node_dbg = format!("{n:?}")
                            .replace("\"", "'")
                            .replace("{", "{\n")
                            .replace("}", "\n}");
                        (format!("node: {}\n{node_dbg}", n.id()), "black")
                    }
                    SplitGraphNodeWeight::ExternalTarget {
                        target, subgraph, ..
                    } => (
                        format!("external target: {target}\nsubgraph: {}", subgraph + 1),
                        "red",
                    ),
                    SplitGraphNodeWeight::GraphRoot { id, .. } => {
                        (format!("graph root: {id}"), "blue")
                    }
                    SplitGraphNodeWeight::SubGraphRoot { id, .. } => {
                        (format!("subgraph root: {id}"), "blue")
                    }
                    SplitGraphNodeWeight::Ordering { id, .. } => {
                        (format!("ordering: {id}"), "green")
                    }
                };

                format!("label = \"{label}\"\ncolor = {color}")
            },
        );

        let home_str = std::env::var("HOME").expect("could not find home directory via env");
        let home = std::path::Path::new(&home_str);

        let mut file =
            std::fs::File::create(home.join(format!("{name}.txt"))).expect("could not create file");
        file.write_all(format!("{dot:?}").as_bytes())
            .expect("could not write file");
    }
}
