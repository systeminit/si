use std::collections::{HashMap, HashSet};
use strum::EnumDiscriminants;

use updates::subgraph_as_updates;

use super::*;

#[derive(Clone, PartialEq, Eq)]
struct TestNodeWeight {
    id: SplitGraphNodeId,
    name: String,
    ordered: bool,
    merkle_tree_hash: MerkleTreeHash,
}

impl TestNodeWeight {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
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
        hasher.update(self.name.as_bytes());
        hasher.update(&[self.ordered as u8]);
        hasher.finalize()
    }

    fn ordered(&self) -> bool {
        self.ordered
    }
}

#[derive(Clone, Debug)]
struct TestReadWriter {
    graphs: HashMap<
        SubGraphAddress,
        SubGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>,
    >,
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
impl SubGraphReader<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>
    for TestReadWriter
{
    type Error = SplitGraphError;

    async fn read_subgraph(
        &self,
        address: SubGraphAddress,
    ) -> Result<
        SubGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>,
        SplitGraphError,
    > {
        self.graphs
            .get(&address)
            .cloned()
            .ok_or(SplitGraphError::SubGraphRead(address, "not found".into()))
    }
}

#[async_trait]
impl SubGraphWriter<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>
    for TestReadWriter
{
    type Error = SplitGraphError;

    async fn write_subgraph(
        &mut self,
        _subgraph: &SubGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>,
    ) -> Result<SubGraphAddress, SplitGraphError> {
        todo!()
    }
}

#[derive(EnumDiscriminants, Clone, Debug, PartialEq, Eq, Hash)]
#[strum_discriminants(derive(Hash))]
pub enum TestEdgeWeight {
    EdgeA,
    EdgeB { is_default: bool },
}

impl EdgeKind for TestEdgeWeightDiscriminants {}

impl CustomEdgeWeight<TestEdgeWeightDiscriminants> for TestEdgeWeight {
    fn kind(&self) -> TestEdgeWeightDiscriminants {
        self.into()
    }

    fn edge_hash(&self) -> Option<ContentHash> {
        let mut hasher = ContentHash::hasher();
        match self {
            TestEdgeWeight::EdgeA => hasher.update(&(1u8.to_le_bytes())),
            TestEdgeWeight::EdgeB { is_default } => {
                hasher.update(&(2u8.to_le_bytes()));
                hasher.update(&[*is_default as u8]);
            }
        }

        Some(hasher.finalize())
    }

    fn clone_as_non_default(&self) -> Self {
        match self {
            TestEdgeWeight::EdgeA => TestEdgeWeight::EdgeA,
            TestEdgeWeight::EdgeB { .. } => TestEdgeWeight::EdgeB { is_default: false },
        }
    }

    fn is_default(&self) -> bool {
        false
    }
}

#[test]
fn ordered_container() -> SplitGraphResult<()> {
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

        splitgraph.add_or_replace_node(container_node)?;
        splitgraph.add_edge(
            splitgraph.root_id()?,
            TestEdgeWeight::EdgeA,
            container_node_id,
        )?;

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
            })?;
            nodes.push(node_id);
            splitgraph.add_edge(container_node_id, TestEdgeWeight::EdgeA, node_id)?;
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
        })?;

        let ordered_children = splitgraph
            .ordered_children(container_id)
            .expect("should have ordered children");

        assert_eq!(reversed_nodes, ordered_children);
    }

    Ok(())
}

#[test]
fn replace_node() -> SplitGraphResult<()> {
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
        splitgraph.add_or_replace_node(node.clone())?;
    }

    for node in nodes.iter_mut() {
        node.name = format!("{}-{}", node.name, node.id);
        splitgraph.add_or_replace_node(node.clone())?;
    }

    for node in &nodes {
        assert_eq!(
            Some(node),
            splitgraph
                .raw_node_weight(node.id())
                .and_then(|n| n.custom())
        );
    }

    Ok(())
}

#[test]
fn cross_graph_edges() -> SplitGraphResult<()> {
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
            })?;
            unsplitgraph.add_or_replace_node(TestNodeWeight {
                id,
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
                ordered,
            })?;
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
                (splitgraph.root_id()?, unsplitgraph.root_id()?)
            } else {
                (
                    name_to_id_map
                        .get(&from_name)
                        .copied()
                        .expect("from name should exist"),
                    name_to_id_map
                        .get(&from_name)
                        .copied()
                        .expect("from name should exist"),
                )
            };

            let to_id = name_to_id_map.get(&to_name).copied().unwrap();

            splitgraph.add_edge(split_from_id, TestEdgeWeight::EdgeA, to_id)?;
            unsplitgraph.add_edge(unsplit_from_id, TestEdgeWeight::EdgeA, to_id)?;

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
                (splitgraph.root_id()?, unsplitgraph.root_id()?)
            } else {
                let id = name_to_id_map
                    .get(&from_name)
                    .copied()
                    .expect("should exist");
                (id, id)
            };

            let outgoing_targets: HashSet<SplitGraphNodeId> = splitgraph
                .edges_directed(split_from_id, Outgoing)?
                .map(|edge_ref| edge_ref.target())
                .collect();
            let unsplit_outgoing_targets: HashSet<SplitGraphNodeId> = unsplitgraph
                .edges_directed(unsplit_from_id, Outgoing)?
                .map(|edge_ref| edge_ref.target())
                .collect();

            let incoming_sources: HashSet<SplitGraphNodeId> = splitgraph
                .edges_directed(split_from_id, Incoming)?
                .map(|edge_ref| edge_ref.source())
                .collect();
            let unsplit_incoming_sources: HashSet<SplitGraphNodeId> = unsplitgraph
                .edges_directed(unsplit_from_id, Incoming)?
                .map(|edge_ref| edge_ref.source())
                .collect();

            if outgoing_targets.is_empty() {
                assert!(expected_outgoing_targets.contains_key(&split_from_id));
                assert!(expected_outgoing_targets.contains_key(&unsplit_from_id));
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
                    if let Some(node) = splitgraph
                        .raw_node_weight(target_id)
                        .and_then(|n| n.custom())
                    {
                        assert_eq!(
                            Some(target_id),
                            name_to_id_map.get(&node.name.as_str()).copied()
                        );
                    }
                }
            }

            if incoming_sources.is_empty() {
                assert!(split_expected_incoming_sources.contains_key(&split_from_id));
                assert!(unsplit_expected_incoming_sources.contains_key(&unsplit_from_id));
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
        splitgraph.remove_edge(
            graph_2_q_id,
            TestEdgeWeightDiscriminants::EdgeA,
            graph_3_t_id,
        )?;
        unsplitgraph.remove_edge(
            graph_2_q_id,
            TestEdgeWeightDiscriminants::EdgeA,
            graph_3_t_id,
        )?;
        splitgraph.cleanup();
        splitgraph.recalculate_merkle_tree_hashes_based_on_touched_nodes();
        unsplitgraph.cleanup();
        unsplitgraph.recalculate_merkle_tree_hashes_based_on_touched_nodes();

        // splitgraph.tiny_dot_to_file("after-removal");

        assert!(splitgraph.raw_node_weight(graph_2_q_id).is_some());
        assert!(unsplitgraph.raw_node_weight(graph_2_q_id).is_some());
        assert!(splitgraph.raw_node_weight(graph_3_s_id).is_some());
        assert!(unsplitgraph.raw_node_weight(graph_3_s_id).is_some());

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
            assert!(splitgraph.raw_node_weight(id).is_none());
            assert!(unsplitgraph.raw_node_weight(id).is_none());
        }
    }

    Ok(())
}

#[test]
fn detect_and_perform_updates_ordered_containers() -> SplitGraphResult<()> {
    let reader_writer = TestReadWriter {
        graphs: HashMap::new(),
    };

    for split_max in [1, 2, 1000] {
        let mut base_graph = SplitGraph::new(&reader_writer, &reader_writer, split_max);
        base_graph.cleanup_and_merkle_tree_hash();
        let mut updated_graph = base_graph.clone();
        updated_graph.cleanup_and_merkle_tree_hash();

        let damaya = TestNodeWeight {
            name: "damaya".to_string(),
            id: Ulid::new(),
            ordered: true,
            merkle_tree_hash: MerkleTreeHash::nil(),
        };

        let evil_earth = TestNodeWeight {
            name: "evil_earth".to_string(),
            id: Ulid::new(),
            ordered: false,
            merkle_tree_hash: MerkleTreeHash::nil(),
        };

        updated_graph.add_or_replace_node(damaya.clone())?;
        updated_graph.add_edge(updated_graph.root_id()?, TestEdgeWeight::EdgeA, damaya.id())?;
        updated_graph.add_or_replace_node(evil_earth.clone())?;
        updated_graph.add_edge(
            updated_graph.root_id()?,
            TestEdgeWeight::EdgeA,
            evil_earth.id(),
        )?;

        let mut name_to_id_map = HashMap::new();
        let children = ["corundum", "uche", "nassun"];
        let mut ordered_child_ids = vec![];
        for name in &children {
            let new_node = TestNodeWeight {
                name: name.to_string(),
                id: Ulid::new(),
                ordered: true,
                merkle_tree_hash: MerkleTreeHash::nil(),
            };
            ordered_child_ids.push(new_node.id());
            updated_graph.add_or_replace_node(new_node.clone())?;
            name_to_id_map.insert(name.to_string(), new_node.id());
            updated_graph.add_edge(
                damaya.id(),
                TestEdgeWeight::EdgeB { is_default: false },
                new_node.id(),
            )?;
            updated_graph.add_edge(evil_earth.id(), TestEdgeWeight::EdgeA, new_node.id())?;
        }

        updated_graph.cleanup_and_merkle_tree_hash();
        let updates = updated_graph.detect_updates(&base_graph);

        assert!(!updates.is_empty());

        base_graph.perform_updates(&updates);
        base_graph.cleanup_and_merkle_tree_hash();

        let updates = updated_graph.detect_updates(&base_graph);
        assert!(updates.is_empty());

        assert_eq!(
            Some(&ordered_child_ids),
            updated_graph.ordered_children(damaya.id()).as_ref()
        );

        assert_eq!(
            Some(&ordered_child_ids),
            base_graph.ordered_children(damaya.id()).as_ref()
        );

        let evil_earth_outgoing_updated: HashSet<_> = updated_graph
            .edges_directed(evil_earth.id(), Outgoing)?
            .map(|edge_ref| edge_ref.target())
            .collect();

        let evil_earth_outgoing_base: HashSet<_> = base_graph
            .edges_directed(evil_earth.id(), Outgoing)?
            .map(|edge_ref| edge_ref.target())
            .collect();

        assert!(evil_earth_outgoing_base
            .difference(&evil_earth_outgoing_updated)
            .next()
            .is_none());

        updated_graph.reorder_node(damaya.id(), |order| order.iter().copied().rev().collect())?;

        let subgraph_for_damaya = updated_graph.subgraph_for_node(damaya.id()).unwrap();
        let updated_root_merkle_before_calculation = updated_graph
            .raw_node_weight(updated_graph.subgraph_root_id(subgraph_for_damaya).unwrap())
            .unwrap()
            .merkle_tree_hash();
        updated_graph.cleanup_and_merkle_tree_hash();
        let updated_root_merkle_after_calculation = updated_graph
            .raw_node_weight(updated_graph.subgraph_root_id(subgraph_for_damaya).unwrap())
            .unwrap()
            .merkle_tree_hash();

        assert_ne!(
            updated_root_merkle_before_calculation,
            updated_root_merkle_after_calculation
        );

        let reversed_ids: Vec<_> = ordered_child_ids.iter().copied().rev().collect();
        assert_eq!(
            Some(&reversed_ids),
            updated_graph.ordered_children(damaya.id()).as_ref()
        );

        let updates_after_reorder = updated_graph.detect_updates(&base_graph);
        assert_eq!(1, updates_after_reorder.len());
        assert!(matches!(
            updates_after_reorder.first().unwrap(),
            Update::ReplaceNode {
                node_weight: SplitGraphNodeWeight::Ordering { .. },
                ..
            }
        ));

        base_graph.perform_updates(&updates_after_reorder);
        base_graph.cleanup_and_merkle_tree_hash();
        assert_eq!(
            Some(&reversed_ids),
            base_graph.ordered_children(damaya.id()).as_ref()
        );
    }

    Ok(())
}

#[test]
fn detect_updates_simple() -> SplitGraphResult<()> {
    let reader_writer = TestReadWriter {
        graphs: HashMap::new(),
    };

    let mut base_graph = SplitGraph::new(&reader_writer, &reader_writer, 3200);
    base_graph.cleanup_and_merkle_tree_hash();
    let mut updated_graph = base_graph.clone();
    updated_graph.cleanup_and_merkle_tree_hash();

    assert!(updated_graph.detect_updates(&base_graph).is_empty());

    let new_node = TestNodeWeight {
        name: "damaya".to_string(),
        id: Ulid::new(),
        ordered: false,
        merkle_tree_hash: MerkleTreeHash::nil(),
    };

    updated_graph.add_or_replace_node(new_node.clone())?;
    updated_graph.add_edge(
        updated_graph.root_id()?,
        TestEdgeWeight::EdgeA,
        new_node.id(),
    )?;
    updated_graph.cleanup_and_merkle_tree_hash();

    let updates = updated_graph.detect_updates(&base_graph);

    assert_eq!(2, updates.len());

    let update_1 = updates.first().unwrap();
    let update_2 = updates.get(1).unwrap();

    assert!(matches!(
        update_1,
        Update::NewNode {
            subgraph_index: 0,
            node_weight: SplitGraphNodeWeight::Custom(TestNodeWeight { .. })
        }
    ));

    let Update::NewNode {
        subgraph_index: 0,
        node_weight: SplitGraphNodeWeight::Custom(custom_node),
    } = update_1
    else {
        unreachable!("we already asserted this!")
    };

    assert_eq!(new_node.node_hash(), custom_node.node_hash());

    assert!(matches!(
        update_2,
        Update::NewEdge {
            subgraph_index: 0,
            edge_weight: SplitGraphEdgeWeight::Custom(TestEdgeWeight::EdgeA),
            ..
        }
    ));

    let Update::NewEdge {
        source,
        destination,
        ..
    } = update_2
    else {
        unreachable!("bridge over the river kwai");
    };

    assert_eq!(updated_graph.root_id()?, *source);
    assert_eq!(new_node.id(), *destination);

    let inverse_updates = base_graph.detect_updates(&updated_graph);
    assert_eq!(2, inverse_updates.len());

    assert!(matches!(
        inverse_updates.first().unwrap(),
        Update::RemoveEdge { .. }
    ));
    assert!(matches!(
        inverse_updates.get(1).unwrap(),
        Update::RemoveNode { .. }
    ));

    let mut second_updated_graph = updated_graph.clone();
    let mut updated_node = new_node.clone();
    updated_node.set_name("syenite".into());
    second_updated_graph.add_or_replace_node(updated_node)?;
    second_updated_graph.cleanup_and_merkle_tree_hash();
    let replace_node_update = second_updated_graph.detect_updates(&updated_graph);
    assert!(matches!(
        replace_node_update.first().unwrap(),
        Update::ReplaceNode {
            subgraph_index: 0,
            node_weight: SplitGraphNodeWeight::Custom(TestNodeWeight { .. }),
        }
    ));

    Ok(())
}

#[test]
fn single_subgraph_as_updates() -> SplitGraphResult<()> {
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

        subgraph.add_edge(
            from_index,
            SplitGraphEdgeWeight::Custom(TestEdgeWeight::EdgeA),
            to_index,
        )?;
    }

    subgraph.cleanup();

    let root_id = subgraph
        .graph
        .node_weight(subgraph.root_index)
        .unwrap()
        .id();

    let expected_edges: Vec<Update<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>> =
        edges
            .into_iter()
            .map(|(source, target)| {
                let source = source
                    .map(|source| node_id_map.get(&source).copied().unwrap())
                    .unwrap_or(root_id);
                let destination = node_id_map.get(&target).copied().unwrap();

                Update::NewEdge {
                    source,
                    destination,
                    edge_weight: SplitGraphEdgeWeight::Custom(TestEdgeWeight::EdgeA),
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

    Ok(())
}
