
use std::{
    collections::{HashMap, HashSet},
    u16,
};

use updates::subgraph_as_updates;

use super::*;

#[derive(Clone, PartialEq, Eq)]
struct TestNodeWeight {
    id: SplitGraphNodeId,
    name: String,
    ordered: bool,
    merkle_tree_hash: MerkleTreeHash,
}

impl std::fmt::Debug for TestNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name = {}", self.name)
    }
}

impl CustomNodeWeight for TestNodeWeight {
    fn id(&self) -> SplitGraphNodeId {
        self.id
    }

    fn lineage_id(&self) -> SplitGraphNodeId {
        self.id
    }

    fn set_merkle_tree_hash(&mut self, hash: MerkleTreeHash) {
        self.merkle_tree_hash = hash;
    }

    fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    fn node_hash(&self) -> ContentHash {
        let mut hasher = ContentHash::hasher();
        hasher.update(&self.id.inner().to_bytes());
        hasher.update(&self.name.as_bytes());
        hasher.update(&[self.ordered as u8]);
        hasher.finalize()
    }

    fn ordered(&self) -> bool {
        self.ordered
    }
}

impl EdgeKind for () {}

struct TestReadWriter {
    graphs: HashMap<SubGraphAddress, SubGraph<TestNodeWeight, (), ()>>,
}

fn add_nodes_to_graph<'a, 'b, E, K>(
    graph: &'a mut SubGraph<TestNodeWeight, E, K>,
    nodes: &'a [&'b str],
    ordered: bool,
) -> HashMap<&'b str, Ulid>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    let mut node_id_map = HashMap::new();
    for node in nodes {
        // "props" here are just nodes that are easy to create and render the name on the dot
        // output. there is no domain modeling in this test.
        let id = SplitGraphNodeId::new();
        let node_weight = TestNodeWeight {
            id,
            name: node.to_string(),
            ordered,
            merkle_tree_hash: MerkleTreeHash::nil(),
        };
        graph.add_node(SplitGraphNodeWeight::Custom(node_weight));

        node_id_map.insert(*node, id);
    }
    node_id_map
}

#[async_trait]
impl SubGraphReader<TestNodeWeight, (), ()> for TestReadWriter {
    type Error = SplitGraphError;

    async fn read_subgraph(
        &self,
        address: SubGraphAddress,
    ) -> Result<SubGraph<TestNodeWeight, (), ()>, SplitGraphError> {
        self.graphs
            .get(&address)
            .cloned()
            .ok_or(SplitGraphError::SubGraphRead(address, "not found".into()))
    }
}

#[async_trait]
impl SubGraphWriter<TestNodeWeight, (), ()> for TestReadWriter {
    type Error = SplitGraphError;

    async fn write_subgraph(
        &mut self,
        _subgraph: &SubGraph<TestNodeWeight, (), ()>,
    ) -> Result<SubGraphAddress, SplitGraphError> {
        todo!()
    }
}

impl CustomEdgeWeight<()> for () {
    fn kind(&self) -> () {
        ()
    }

    fn edge_hash(&self) -> Option<ContentHash> {
        None
    }
}

#[test]
fn ordered_container() {
    let reader_writer = TestReadWriter {
        graphs: HashMap::new(),
    };
    let mut splitgraph = SplitGraph::new(&reader_writer, &reader_writer, 10000);

    let mut node_name_to_id_map = HashMap::new();
    let container_nodes: Vec<TestNodeWeight> = ["a"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
            ordered: true,
        })
        .collect();

    let mut nodes_per_container = HashMap::new();

    for container_node in container_nodes {
        let container_name = container_node.name.to_owned();
        let container_node_id = container_node.id();

        splitgraph.add_or_replace_node(container_node);
        splitgraph.add_edge(splitgraph.root_id(), (), container_node_id);

        node_name_to_id_map.insert(container_name.to_owned(), container_node_id);
        let mut nodes = vec![];
        for i in 0..5 {
            let node_name = format!("{container_name}-{i}");
            let node_id = Ulid::new();
            node_name_to_id_map.insert(node_name.to_owned(), node_id);
            splitgraph.add_or_replace_node(TestNodeWeight {
                id: node_id,
                name: node_name,
                ordered: false,
                merkle_tree_hash: MerkleTreeHash::nil(),
            });
            nodes.push(node_id);
            splitgraph.add_edge(container_node_id, (), node_id);
        }
        nodes_per_container.insert(container_node_id, nodes);
    }

    for (container_id, expected_nodes) in nodes_per_container {
        let ordered_children = splitgraph
            .ordered_children(container_id)
            .expect("should have ordered children");
        assert_eq!(expected_nodes, ordered_children);

        let reversed_nodes: Vec<_> = expected_nodes.into_iter().rev().collect();
        splitgraph.reorder_node(container_id, |current_order| {
            current_order
                .iter()
                .rev()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        });

        let ordered_children = splitgraph
            .ordered_children(container_id)
            .expect("should have ordered children");

        assert_eq!(reversed_nodes, ordered_children);
    }
}

#[test]
fn replace_node() {
    let reader_writer = TestReadWriter {
        graphs: HashMap::new(),
    };
    let mut splitgraph = SplitGraph::new(&reader_writer, &reader_writer, 2);

    let mut nodes: Vec<TestNodeWeight> = ["1", "2", "3", "4", "5", "6"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
            ordered: false,
        })
        .collect();

    for node in &nodes {
        splitgraph.add_or_replace_node(node.clone());
    }

    for node in nodes.iter_mut() {
        node.name = format!("{}-{}", node.name, node.id);
        splitgraph.add_or_replace_node(node.clone());
    }

    for node in &nodes {
        assert_eq!(
            Some(node),
            splitgraph.node_weight(node.id()).and_then(|n| n.custom())
        );
    }
}

#[test]
fn cross_graph_edges() {
    for ordered in [false, true] {
        let reader_writer = TestReadWriter {
            graphs: HashMap::new(),
        };

        let mut splitgraph = SplitGraph::new(&reader_writer, &reader_writer, 9);
        let mut unsplitgraph = SplitGraph::new(&reader_writer, &reader_writer, MAX_NODES as u16);

        let nodes = [
            "graph-1-a",
            "graph-1-b",
            "graph-1-c",
            "graph-1-d",
            "graph-1-e",
            "graph-1-f",
            "graph-1-g",
            "graph-1-h",
            "graph-1-i",
            "graph-2-j",
            "graph-2-k",
            "graph-2-l",
            "graph-2-m",
            "graph-2-n",
            "graph-2-o",
            "graph-2-p",
            "graph-2-q",
            "graph-2-r",
            "graph-3-s",
            "graph-3-t",
            "graph-3-u",
            "graph-3-v",
            "graph-3-w",
            "graph-3-x",
            "graph-3-y",
            "graph-3-z",
        ];

        let edges = [
            ("", "graph-1-a"),
            ("graph-1-a", "graph-1-b"),
            ("graph-1-a", "graph-1-c"),
            ("graph-1-c", "graph-1-d"),
            ("graph-1-d", "graph-1-e"),
            ("graph-1-e", "graph-1-f"),
            ("graph-1-f", "graph-1-g"),
            ("graph-1-g", "graph-1-h"),
            ("graph-1-h", "graph-1-i"),
            ("graph-1-a", "graph-2-j"),
            ("graph-1-a", "graph-2-k"),
            ("graph-1-b", "graph-2-k"),
            ("graph-1-c", "graph-2-k"),
            ("", "graph-2-l"),
            ("graph-2-l", "graph-1-b"),
            ("graph-2-l", "graph-1-c"),
            ("graph-2-l", "graph-1-d"),
            ("graph-2-l", "graph-2-m"),
            ("graph-2-l", "graph-2-n"),
            ("graph-2-l", "graph-2-o"),
            ("graph-2-l", "graph-2-p"),
            ("graph-2-p", "graph-2-q"),
            ("graph-2-q", "graph-2-r"),
            ("graph-2-q", "graph-3-s"),
            ("graph-2-q", "graph-3-t"),
            ("graph-3-t", "graph-1-b"),
            ("graph-3-t", "graph-3-u"),
            ("graph-3-t", "graph-3-v"),
            ("graph-3-t", "graph-3-w"),
            ("graph-3-t", "graph-3-x"),
            ("graph-3-t", "graph-3-y"),
            ("graph-3-t", "graph-3-z"),
        ];

        let mut name_to_id_map = HashMap::new();
        for name in &nodes {
            let id = Ulid::new();
            splitgraph.add_or_replace_node(TestNodeWeight {
                id,
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
                ordered,
            });
            unsplitgraph.add_or_replace_node(TestNodeWeight {
                id,
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
                ordered,
            });
            println!(
                "added node {name}:{id}, subgraphs: {}",
                splitgraph.subgraph_count()
            );
            name_to_id_map.insert(name, id);
        }

        let mut expected_outgoing_targets: HashMap<SplitGraphNodeId, HashSet<SplitGraphNodeId>> =
            HashMap::new();
        let mut split_expected_incoming_sources: HashMap<
            SplitGraphNodeId,
            HashSet<SplitGraphNodeId>,
        > = HashMap::new();
        let mut unsplit_expected_incoming_sources: HashMap<
            SplitGraphNodeId,
            HashSet<SplitGraphNodeId>,
        > = HashMap::new();

        for (from_name, to_name) in edges {
            let (split_from_id, unsplit_from_id) = if from_name.is_empty() {
                (splitgraph.root_id(), unsplitgraph.root_id())
            } else {
                (
                    name_to_id_map.get(&from_name).copied().unwrap(),
                    name_to_id_map.get(&from_name).copied().unwrap(),
                )
            };

            let to_id = name_to_id_map.get(&to_name).copied().unwrap();

            println!("adding edge {from_name}:{split_from_id} -> {to_name}:{to_id}");

            splitgraph.add_edge(split_from_id, (), to_id);
            println!("adding to unsplitgraph");
            unsplitgraph.add_edge(unsplit_from_id, (), to_id);

            expected_outgoing_targets
                .entry(split_from_id)
                .and_modify(|outgoing| {
                    outgoing.insert(to_id);
                })
                .or_insert(HashSet::from([to_id]));
            expected_outgoing_targets
                .entry(unsplit_from_id)
                .and_modify(|outgoing| {
                    outgoing.insert(to_id);
                })
                .or_insert(HashSet::from([to_id]));

            split_expected_incoming_sources
                .entry(to_id)
                .and_modify(|incoming| {
                    incoming.insert(split_from_id);
                })
                .or_insert(HashSet::from([split_from_id]));
            unsplit_expected_incoming_sources
                .entry(to_id)
                .and_modify(|incoming| {
                    incoming.insert(unsplit_from_id);
                })
                .or_insert(HashSet::from([unsplit_from_id]));
        }

        for from_name in &nodes {
            let (split_from_id, unsplit_from_id) = if from_name.is_empty() {
                (splitgraph.root_id(), unsplitgraph.root_id())
            } else {
                let id = name_to_id_map.get(&from_name).copied().unwrap();
                (id, id)
            };

            let outgoing_targets: HashSet<SplitGraphNodeId> = splitgraph
                .edges_directed(split_from_id, Outgoing)
                .map(|edge_ref| edge_ref.target())
                .collect();
            let unsplit_outgoing_targets: HashSet<SplitGraphNodeId> = unsplitgraph
                .edges_directed(unsplit_from_id, Outgoing)
                .map(|edge_ref| edge_ref.target())
                .collect();

            let incoming_sources: HashSet<SplitGraphNodeId> = splitgraph
                .edges_directed(split_from_id, Incoming)
                .map(|edge_ref| edge_ref.source())
                .collect();
            let unsplit_incoming_sources: HashSet<SplitGraphNodeId> = unsplitgraph
                .edges_directed(unsplit_from_id, Incoming)
                .map(|edge_ref| edge_ref.source())
                .collect();

            let name = splitgraph
                .node_weight(split_from_id)
                .and_then(|n| n.custom().map(|n| n.name.as_str()))
                .unwrap();

            println!(
                "{split_from_id} ({name}):\n\t{:?}\n\t{:?}",
                outgoing_targets, incoming_sources
            );

            if outgoing_targets.is_empty() {
                assert!(expected_outgoing_targets.get(&split_from_id).is_none());
                assert!(expected_outgoing_targets.get(&unsplit_from_id).is_none());
            } else {
                assert_eq!(
                    expected_outgoing_targets
                        .get(&split_from_id)
                        .cloned()
                        .unwrap(),
                    outgoing_targets
                );
                assert_eq!(
                    expected_outgoing_targets
                        .get(&unsplit_from_id)
                        .cloned()
                        .unwrap(),
                    unsplit_outgoing_targets
                );

                for target_id in outgoing_targets {
                    if let Some(node) = splitgraph.node_weight(target_id).and_then(|n| n.custom()) {
                        assert_eq!(
                            Some(target_id),
                            name_to_id_map.get(&node.name.as_str()).copied()
                        );
                    }
                }
            }

            if incoming_sources.is_empty() {
                assert!(split_expected_incoming_sources
                    .get(&split_from_id)
                    .is_none());
                assert!(unsplit_expected_incoming_sources
                    .get(&unsplit_from_id)
                    .is_none());
            } else {
                assert_eq!(
                    split_expected_incoming_sources
                        .get(&split_from_id)
                        .cloned()
                        .unwrap(),
                    incoming_sources
                );
                assert_eq!(
                    unsplit_expected_incoming_sources
                        .get(&unsplit_from_id)
                        .cloned()
                        .unwrap(),
                    unsplit_incoming_sources
                );
            }
        }

        // splitgraph.tiny_dot_to_file("before-removal");
        // unsplitgraph.tiny_dot_to_file("unsplitgraph");

        let graph_2_q = "graph-2-q";
        let graph_3_t = "graph-3-t";
        let graph_3_s = "graph-3-s";
        let graph_2_q_id = name_to_id_map.get(&graph_2_q).copied().unwrap();
        let graph_3_t_id = name_to_id_map.get(&graph_3_t).copied().unwrap();
        let graph_3_s_id = name_to_id_map.get(&graph_3_s).copied().unwrap();
        splitgraph.remove_edge(graph_2_q_id, (), graph_3_t_id);
        unsplitgraph.remove_edge(graph_2_q_id, (), graph_3_t_id);
        splitgraph.cleanup();
        splitgraph.recalculate_merkle_tree_hashes_based_on_touched_nodes();
        unsplitgraph.cleanup();
        unsplitgraph.recalculate_merkle_tree_hashes_based_on_touched_nodes();

        // splitgraph.tiny_dot_to_file("after-removal");

        assert!(splitgraph.node_weight(graph_2_q_id).is_some());
        assert!(unsplitgraph.node_weight(graph_2_q_id).is_some());
        assert!(splitgraph.node_weight(graph_3_s_id).is_some());
        assert!(unsplitgraph.node_weight(graph_3_s_id).is_some());

        for graph_3_name in [
            "graph-3-t",
            "graph-3-u",
            "graph-3-v",
            "graph-3-w",
            "graph-3-x",
            "graph-3-y",
            "graph-3-z",
        ] {
            let id = name_to_id_map.get(&graph_3_name).copied().unwrap();
            assert!(splitgraph.node_weight(id).is_none());
            assert!(unsplitgraph.node_weight(id).is_none());
        }
    }
}

#[test]
fn perform_updates() {}

#[test]
fn single_subgraph_as_updates() {
    let nodes = ["r", "t", "u", "v", "a", "b", "c", "d", "e"];
    let edges = [
        (None, "r"),
        (None, "t"),
        (Some("t"), "u"),
        (Some("u"), "v"),
        (Some("r"), "a"),
        (Some("r"), "b"),
        (Some("r"), "e"),
        (Some("a"), "c"),
        (Some("b"), "c"),
        (Some("c"), "d"),
        (Some("b"), "d"),
        (Some("d"), "e"),
        (Some("c"), "e"),
        (Some("e"), "u"),
        (Some("c"), "u"),
        (Some("a"), "u"),
        (Some("a"), "b"),
    ];

    let mut subgraph = SubGraph::new_with_root();
    let node_id_map = add_nodes_to_graph(&mut subgraph, &nodes, false);

    for (source, target) in edges {
        let from_index = match source {
            Some(name) => subgraph
                .node_id_to_index(node_id_map.get(&name).copied().unwrap())
                .unwrap(),
            None => subgraph.root_index,
        };
        let to_index = subgraph
            .node_id_to_index(node_id_map.get(&target).copied().unwrap())
            .unwrap();

        subgraph.add_edge(from_index, SplitGraphEdgeWeight::Custom(()), to_index);
    }

    subgraph.cleanup();

    let root_id = subgraph
        .graph
        .node_weight(subgraph.root_index)
        .unwrap()
        .id();

    let expected_edges: Vec<Update<TestNodeWeight, (), ()>> = edges
        .into_iter()
        .map(|(source, target)| {
            let source = source
                .map(|source| node_id_map.get(&source).copied().unwrap())
                .unwrap_or(root_id);
            let destination = node_id_map.get(&target).copied().unwrap();

            Update::NewEdge {
                source,
                destination,
                edge_weight: SplitGraphEdgeWeight::Custom(()),
                subgraph_index: 0,
            }
        })
        .collect();

    let updates = subgraph_as_updates(&subgraph, 0);
    assert!(!updates.is_empty());
    let mut new_edge_count = 0;
    let mut new_node_count = 0;
    for update in updates {
        match &update {
            new_edge @ Update::NewEdge { .. } => {
                new_edge_count += 1;
                assert!(expected_edges.contains(new_edge));
            }
            Update::NewNode {
                node_weight: SplitGraphNodeWeight::Custom(TestNodeWeight { id, name, .. }),
                ..
            } => {
                new_node_count += 1;
                assert_eq!(node_id_map.get(&name.as_str()), Some(id))
            }
            _ => {}
        }
    }

    assert_eq!(expected_edges.len(), new_edge_count);
    assert_eq!(nodes.len(), new_node_count);
}
