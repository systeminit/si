use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

use crate::{
    CustomEdgeWeight, CustomNodeWeight, EdgeKind, NodeId, SplitGraphEdgeWeight,
    SplitGraphNodeWeight, MAX_NODES,
};

pub type SubGraphNodeIndex = NodeIndex<u16>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(super) graph: StableDiGraph<SplitGraphNodeWeight<N>, SplitGraphEdgeWeight<E, K>, u16>,
    pub(super) node_index_by_id: HashMap<NodeId, SubGraphNodeIndex>,
    pub(super) root_index: SubGraphNodeIndex,
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

    pub fn tiny_dot_to_file(&self, name: &str) {
        let dot = petgraph::dot::Dot::with_attr_getters(
            &self.graph,
            &[
                petgraph::dot::Config::NodeNoLabel,
                petgraph::dot::Config::EdgeNoLabel,
            ],
            &|_, edge_ref| match edge_ref.weight() {
                SplitGraphEdgeWeight::Custom(_) => format!("label = \"\"\ncolor = black"),
                SplitGraphEdgeWeight::ExternalSource {
                    source_id,
                    subgraph,
                    ..
                } => {
                    format!(
                        "label = \"external source: {source_id}\nsubgraph: {}\"\ncolor = red",
                        subgraph + 1
                    )
                }
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
                    SplitGraphNodeWeight::GraphRoot(_) => (format!("graph root"), "blue"),
                    SplitGraphNodeWeight::SubGraphRoot(_) => (format!("subgraph root"), "blue"),
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
