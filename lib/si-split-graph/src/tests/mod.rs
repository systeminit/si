use std::collections::{
    HashMap,
    HashSet,
};

use petgraph::visit::{
    Control,
    DfsEvent,
    IntoNeighbors,
    IntoNodeIdentifiers,
};
use strum::EnumDiscriminants;
use updates::subgraph_as_updates;

use super::*;

#[derive(Clone, PartialEq, Eq, Hash)]
struct TestNodeWeight {
    id: SplitGraphNodeId,
    lineage_id: SplitGraphNodeId,
    name: String,
    merkle_tree_hash: MerkleTreeHash,
}

impl TestNodeWeight {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

impl std::fmt::Debug for TestNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id = {:?},\nname = {}", self.id, self.name)
    }
}

impl NodeKind for () {}

impl CustomNodeWeight for TestNodeWeight {
    type Kind = ();

    fn kind(&self) -> Self::Kind {}

    fn id(&self) -> SplitGraphNodeId {
        self.id
    }

    fn set_id(&mut self, id: SplitGraphNodeId) {
        self.id = id;
    }

    fn lineage_id(&self) -> SplitGraphNodeId {
        self.lineage_id
    }

    fn set_lineage_id(&mut self, id: SplitGraphNodeId) {
        self.lineage_id = id;
    }

    fn dot_details(&self) -> String {
        self.name.clone()
    }

    fn entity_kind(&self) -> EntityKind {
        EntityKind::Component
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
        hasher.finalize()
    }
}

fn add_edges_to_splitgraph<'a, 'b, E, K>(
    graph: &'a mut SplitGraph<TestNodeWeight, E, K>,
    edges: &'a [(Option<&'b str>, E, &'b str, bool)],
    node_id_map: &'a HashMap<&'b str, Ulid>,
) where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    for (source, edge, target, ordered) in edges {
        let from_id = match source {
            Some(source) => node_id_map
                .get(source)
                .copied()
                .expect("source should be in map"),
            None => graph.root_id().expect("should have a root id"),
        };

        let to_id = node_id_map
            .get(target)
            .copied()
            .expect("target should be in map");

        if *ordered {
            graph
                .add_ordered_edge(from_id, edge.clone(), to_id)
                .expect("add ordered edge");
        } else {
            graph
                .add_edge(from_id, edge.clone(), to_id)
                .expect("add edge");
        }
    }
}

fn add_nodes_to_splitgraph<'a, 'b, E, K>(
    graph: &'a mut SplitGraph<TestNodeWeight, E, K>,
    nodes: &'a [&'b str],
) -> HashMap<&'b str, Ulid>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    let mut node_id_map = HashMap::new();

    for &node in nodes {
        let id = SplitGraphNodeId::new();
        let node_weight = TestNodeWeight {
            id,
            lineage_id: SplitGraphNodeId::new(),
            name: node.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
        };
        graph
            .add_or_replace_node(node_weight)
            .expect("add_or_replace_node");

        node_id_map.insert(node, id);
    }

    node_id_map
}

fn add_nodes_to_subgraph<'a, 'b, E, K>(
    graph: &'a mut SubGraph<TestNodeWeight, E, K>,
    nodes: &'a [&'b str],
) -> HashMap<&'b str, Ulid>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    let mut node_id_map = HashMap::new();

    for &node in nodes {
        let id = SplitGraphNodeId::new();
        let node_weight = TestNodeWeight {
            id,
            lineage_id: SplitGraphNodeId::new(),
            name: node.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
        };
        graph.add_node(SplitGraphNodeWeight::Custom(node_weight));

        node_id_map.insert(node, id);
    }

    node_id_map
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

    fn edge_entropy(&self) -> Option<Vec<u8>> {
        Some(match self {
            TestEdgeWeight::EdgeA => 1u8.to_le_bytes().to_vec(),
            TestEdgeWeight::EdgeB { is_default } => [*is_default as u8].to_vec(),
        })
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
fn ordered_edges() -> SplitGraphResult<()> {
    let mut splitgraph = SplitGraph::new(10000);

    let mut node_name_to_id_map = HashMap::new();
    let container_nodes: Vec<TestNodeWeight> = ["a"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
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
                lineage_id: SplitGraphNodeId::new(),
                name: node_name,
                merkle_tree_hash: MerkleTreeHash::nil(),
            })?;
            nodes.push(node_id);
            splitgraph.add_ordered_edge(container_node_id, TestEdgeWeight::EdgeA, node_id)?;
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

// #[test]
// fn cross_graph_external_source_deep_removals() -> SplitGraphResult<()> {
// let mut split_graph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> = SplitGraph::new(1);

//     let nodes: Vec<_> = ["a", "b", "c", "d", "e", "f"]
//         .into_iter()
//         .map(|name| TestNodeWeight {
//             id: Ulid::new(),
//             lineage_id: SplitGraphNodeId::new(),
//             name: name.to_string(),
//             merkle_tree_hash: MerkleTreeHash::nil(),
//         })
//         .collect();
// let mut node_id_map = HashMap::new();
//    let mut last_node = None;
//    for node in &nodes {
//        split_graph.add_or_replace_node(node.clone())?;
//        node_id_map.insert(node.name.as_str(), node.id);
//        if node.name == "a" {
//            split_graph.add_edge(split_graph.root_id()?, TestEdgeWeight::EdgeA, node.id)?;
//        }
//        if let Some(last_node_id) = last_node {
//            split_graph.add_edge(last_node_id, TestEdgeWeight::EdgeA, node.id)?;
//        }
//        last_node = Some(node.id);
//    }

//     let new_root_id = Ulid::new();
//     split_graph.add_or_replace_node(TestNodeWeight {
//         id: new_root_id,
//         lineage_id: Ulid::new(),
//         name: "replacement".into(),
//         merkle_tree_hash: MerkleTreeHash::nil(),
//     })?;
//     split_graph.add_edge(split_graph.root_id()?, TestEdgeWeight::EdgeA, new_root_id)?;

//     Ok(())
// }
//

#[test]
fn default_edges() -> SplitGraphResult<()> {
    let mut split_graph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> =
        SplitGraph::new(3);

    let nodes: Vec<_> = ["a", "b", "c", "d", "e", "f"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
        })
        .collect();

    let mut node_id_map = HashMap::new();
    for node in &nodes {
        split_graph.add_or_replace_node(node.clone())?;
        node_id_map.insert(node.name.as_str(), node.id);
        if node.name == "a" {
            split_graph.add_edge(split_graph.root_id()?, TestEdgeWeight::EdgeA, node.id)?;
        } else {
            split_graph.add_edge(
                node_id_map.get(&"a").copied().unwrap(),
                TestEdgeWeight::EdgeB { is_default: false },
                node.id,
            )?;
        }
    }

    let a_id = node_id_map.get(&"a").copied().unwrap();
    let b_id = node_id_map.get(&"b").copied().unwrap();

    split_graph.cleanup_and_merkle_tree_hash();
    let mut updated_graph = split_graph.clone();
    updated_graph.remove_edge(a_id, TestEdgeWeightDiscriminants::EdgeB, b_id)?;
    updated_graph.add_edge(a_id, TestEdgeWeight::EdgeB { is_default: true }, b_id)?;
    updated_graph.cleanup_and_merkle_tree_hash();

    fn default_edge_assertions(
        graph: &SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>,
        from_id: SplitGraphNodeId,
        default_target: SplitGraphNodeId,
    ) -> SplitGraphResult<()> {
        let edges: Vec<_> = graph
            .edges_directed(from_id, Outgoing)?
            .map(|edge| (edge.weight().clone(), edge.target()))
            .collect();

        assert_eq!(5, edges.len());
        let mut hit_default = false;
        for edge in edges {
            if edge.1 != default_target {
                assert!(matches!(
                    edge.0,
                    TestEdgeWeight::EdgeB { is_default: false }
                ));
            } else {
                assert!(
                    matches!(edge.0, TestEdgeWeight::EdgeB { is_default: true }),
                    "{default_target:?} should be default target"
                );
                hit_default = true;
            }
        }

        assert!(hit_default);
        Ok(())
    }

    default_edge_assertions(&updated_graph, a_id, b_id)?;

    let updates = split_graph.detect_updates(&updated_graph);
    split_graph.perform_updates(&updates);
    split_graph.cleanup_and_merkle_tree_hash();
    default_edge_assertions(&split_graph, a_id, b_id)?;

    Ok(())
}

#[test]
fn cross_graph_node_id_updates() -> SplitGraphResult<()> {
    let mut split_graph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> =
        SplitGraph::new(1);

    let nodes: Vec<_> = ["a", "b", "c", "d", "e", "f"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
        })
        .collect();

    let mut node_id_map = HashMap::new();
    for node in &nodes {
        node_id_map.insert(node.name.as_str(), node.id());
        split_graph.add_or_replace_node(node.clone())?;
        let subgraph = split_graph.subgraph_mut_for_node(node.id()).unwrap();
        let index = subgraph.node_id_to_index(node.id()).unwrap();
        subgraph.add_edge(
            subgraph.root_index,
            SplitGraphEdgeWeight::Custom(TestEdgeWeight::EdgeA),
            index,
        );
    }

    split_graph.cleanup_and_merkle_tree_hash();
    let mut split_graph_with_node_id_update = split_graph.clone();

    let a_id = node_id_map.get(&"a").copied().unwrap();
    let b_id = node_id_map.get(&"b").copied().unwrap();
    let a_lineage_id = split_graph_with_node_id_update
        .node_weight(a_id)
        .unwrap()
        .lineage_id();
    let f_id = node_id_map.get(&"f").copied().unwrap();

    split_graph_with_node_id_update.remove_node(a_id)?;
    split_graph_with_node_id_update.update_node_id(dbg!(f_id), dbg!(a_id), a_lineage_id)?;
    split_graph_with_node_id_update.cleanup_and_merkle_tree_hash();
    assert!(split_graph_with_node_id_update.node_weight(a_id).is_some());

    let updates = split_graph.detect_updates(&split_graph_with_node_id_update);
    split_graph.perform_updates(&updates);
    split_graph.cleanup_and_merkle_tree_hash();

    assert!(split_graph.node_weight(a_id).is_some());
    assert!(split_graph.node_weight(f_id).is_none());

    split_graph_with_node_id_update.remove_node(a_id)?;
    split_graph_with_node_id_update.update_node_id(dbg!(b_id), dbg!(a_id), a_lineage_id)?;

    split_graph_with_node_id_update.cleanup_and_merkle_tree_hash();
    assert!(split_graph_with_node_id_update.node_weight(a_id).is_some());
    assert!(split_graph_with_node_id_update.node_weight(b_id).is_none());

    let updates = split_graph.detect_updates(&split_graph_with_node_id_update);
    dbg!(&updates);
    split_graph.perform_updates(&updates);
    split_graph.cleanup_and_merkle_tree_hash();

    dbg!(split_graph.node_id_to_index(a_id));

    assert!(split_graph.node_weight(a_id).is_some());
    assert!(split_graph.node_weight(b_id).is_none());

    Ok(())
}

#[test]
fn external_source_many_to_one_removal_and_id_updates() -> SplitGraphResult<()> {
    let mut split_graph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> =
        SplitGraph::new(3);

    let first_graph_nodes: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
        })
        .collect();

    for node in &first_graph_nodes {
        split_graph.add_or_replace_node(node.clone())?;
    }

    let second_graph_nodes: Vec<_> = ["d", "e", "f"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
        })
        .collect();

    for node in &second_graph_nodes {
        split_graph.add_or_replace_node(node.clone())?;
        let subgraph = split_graph.subgraphs.get_mut(1).unwrap();
        let index = subgraph.node_id_to_index(node.id()).unwrap();
        subgraph.add_edge(
            subgraph.root_index,
            SplitGraphEdgeWeight::Custom(TestEdgeWeight::EdgeA),
            index,
        );
    }

    for second_graph_node in &second_graph_nodes {
        for first_graph_node in &first_graph_nodes {
            split_graph.add_edge(
                second_graph_node.id,
                TestEdgeWeight::EdgeA,
                first_graph_node.id,
            )?;
            split_graph.add_edge(
                second_graph_node.id,
                TestEdgeWeight::EdgeB { is_default: false },
                first_graph_node.id,
            )?;
        }
    }

    for first_graph_node in &first_graph_nodes {
        let subgraph = split_graph.subgraphs().first().unwrap();
        let index = subgraph
            .node_index_by_id
            .get(&first_graph_node.id)
            .copied()
            .unwrap();

        let incoming_edges: Vec<_> = subgraph
            .graph
            .edges_directed(index, Incoming)
            .filter(|edge| match edge.weight() {
                SplitGraphEdgeWeight::ExternalSource { source_id, .. } => second_graph_nodes
                    .iter()
                    .any(|node| node.id() == *source_id),
                _ => false,
            })
            .map(|edge| edge.weight().clone())
            .collect();

        assert_eq!(incoming_edges.len(), 6);
    }

    split_graph.cleanup_and_merkle_tree_hash();
    let mut split_graph_with_deletes = split_graph.clone();

    for first_graph_node in &first_graph_nodes {
        let subgraph = split_graph_with_deletes.subgraphs().first().unwrap();
        let index = subgraph
            .node_index_by_id
            .get(&first_graph_node.id)
            .copied()
            .unwrap();

        let incoming_edges: Vec<_> = subgraph
            .graph
            .edges_directed(index, Incoming)
            .filter(|edge| match edge.weight() {
                SplitGraphEdgeWeight::ExternalSource { source_id, .. } => second_graph_nodes
                    .iter()
                    .any(|node| node.id() == *source_id),
                _ => false,
            })
            .map(|edge| edge.weight().clone())
            .collect();

        assert_eq!(incoming_edges.len(), 6);
    }

    let d_id = second_graph_nodes.first().unwrap().id();
    split_graph_with_deletes.remove_node(d_id)?;
    split_graph_with_deletes.cleanup_and_merkle_tree_hash();

    let mut updated_split_graph = split_graph.clone();
    updated_split_graph.cleanup_and_merkle_tree_hash();
    let updates = updated_split_graph.detect_updates(&split_graph_with_deletes);
    updated_split_graph.perform_updates(&updates);
    updated_split_graph.cleanup_and_merkle_tree_hash();

    for first_graph_node in &first_graph_nodes {
        let subgraph_1 = split_graph_with_deletes.subgraphs().first().unwrap();
        let subgraph_2 = updated_split_graph.subgraphs().first().unwrap();

        for subgraph in [subgraph_1, subgraph_2] {
            let index = subgraph
                .node_index_by_id
                .get(&first_graph_node.id)
                .copied()
                .unwrap();

            let incoming_edges: Vec<_> = subgraph
                .graph
                .edges_directed(index, Incoming)
                .filter(|edge| match edge.weight() {
                    SplitGraphEdgeWeight::ExternalSource { source_id, .. } => second_graph_nodes
                        .iter()
                        .any(|node| node.id() == *source_id),
                    _ => false,
                })
                .map(|edge| edge.weight().clone())
                .collect();

            assert_eq!(incoming_edges.len(), 4);
        }
    }

    let e_id = second_graph_nodes.get(1).unwrap().id();
    let e_lineage_id = second_graph_nodes.get(1).unwrap().lineage_id();
    let a_id = first_graph_nodes.first().unwrap().id();
    let a_lineage_id = first_graph_nodes.first().unwrap().lineage_id();
    split_graph_with_deletes.remove_edge(e_id, TestEdgeWeightDiscriminants::EdgeB, a_id)?;
    split_graph_with_deletes.cleanup_and_merkle_tree_hash();

    let updates = updated_split_graph.detect_updates(&split_graph_with_deletes);
    updated_split_graph.perform_updates(&updates);

    let incoming_to_a: Vec<_> = split_graph_with_deletes
        .edges_directed(a_id, Incoming)?
        .map(|edge| (edge.weight().clone(), edge.source()))
        .collect();

    assert_eq!(3, incoming_to_a.len());

    for (_, source) in incoming_to_a {
        assert_ne!(source, d_id);
        assert!(second_graph_nodes.iter().any(|node| node.id() == source));
    }

    let incoming_to_a: Vec<_> = updated_split_graph
        .edges_directed(a_id, Incoming)?
        .map(|edge| (edge.weight().clone(), edge.source()))
        .collect();

    assert_eq!(3, incoming_to_a.len());

    // update a_id
    let new_id = Ulid::new();
    let external_targets_a = {
        let subgraph_1 = split_graph_with_deletes.subgraphs().get(1).unwrap();
        let external_targets_a: Vec<_> = subgraph_1
            .nodes()
            .filter(|node| match node {
                SplitGraphNodeWeight::ExternalTarget { target, .. } => *target == a_id,
                _ => false,
            })
            .cloned()
            .collect();
        external_targets_a
    };

    assert_eq!(3, external_targets_a.len());

    split_graph_with_deletes.update_node_id(dbg!(a_id), dbg!(new_id), a_lineage_id)?;
    split_graph_with_deletes.cleanup_and_merkle_tree_hash();
    assert!(split_graph_with_deletes.node_weight(a_id).is_none());
    assert!(split_graph_with_deletes.node_weight(new_id).is_some());
    updated_split_graph.cleanup_and_merkle_tree_hash();
    let updates = updated_split_graph.detect_updates(&split_graph_with_deletes);
    updated_split_graph.perform_updates(&updates);
    assert!(updated_split_graph.node_weight(new_id).is_some());
    updated_split_graph.cleanup_and_merkle_tree_hash();
    assert!(updated_split_graph.node_weight(a_id).is_none());
    assert!(updated_split_graph.node_weight(new_id).is_some());

    {
        let subgraph_1 = split_graph_with_deletes.subgraphs().get(1).unwrap();
        let subgraph_1_updated = updated_split_graph.subgraphs().get(1).unwrap();
        for subgraph in [subgraph_1, subgraph_1_updated] {
            let external_targets: Vec<_> = subgraph
                .nodes()
                .filter(|node| {
                    external_targets_a
                        .iter()
                        .any(|ext_target_a| ext_target_a.id() == node.id())
                })
                .filter(|node| node.external_target_id() == Some(new_id))
                .cloned()
                .collect();
            assert_eq!(external_targets_a.len(), external_targets.len());
        }
    }

    // update e_id id and verify source_id edge updates

    let external_source_id_old_e_before_changes: Vec<_> = split_graph_with_deletes
        .subgraphs()
        .first()
        .unwrap()
        .edges()
        .map(|(edge, _, _)| edge)
        .filter(|edge| edge.external_source_data().map(|data| data.source_id) == Some(e_id))
        .cloned()
        .collect();

    dbg!(external_source_id_old_e_before_changes.len());

    let new_e_id = Ulid::new();
    split_graph_with_deletes.update_node_id(e_id, new_e_id, e_lineage_id)?;
    split_graph_with_deletes.cleanup_and_merkle_tree_hash();
    assert!(split_graph_with_deletes.node_weight(e_id).is_none());
    assert!(split_graph_with_deletes.node_weight(new_e_id).is_some());
    updated_split_graph.cleanup_and_merkle_tree_hash();
    let updates = updated_split_graph.detect_updates(&split_graph_with_deletes);
    updated_split_graph.perform_updates(&updates);
    assert!(updated_split_graph.node_weight(new_e_id).is_some());
    updated_split_graph.cleanup_and_merkle_tree_hash();
    assert!(updated_split_graph.node_weight(e_id).is_none());
    assert!(updated_split_graph.node_weight(new_e_id).is_some());

    {
        let subgraph_0 = split_graph_with_deletes.subgraphs().first().unwrap();
        let subgraph_0_updated = updated_split_graph.subgraphs().first().unwrap();
        for subgraph in [subgraph_0, subgraph_0_updated] {
            let external_source_id_new_e: Vec<_> = subgraph
                .edges()
                .map(|(edge, _, _)| edge)
                .filter(|edge| {
                    edge.external_source_data().map(|data| data.source_id) == Some(new_e_id)
                })
                .cloned()
                .collect();

            assert_eq!(
                external_source_id_old_e_before_changes.len(),
                external_source_id_new_e.len()
            );

            let external_source_id_old_e: Vec<_> = subgraph
                .edges()
                .map(|(edge, _, _)| edge)
                .filter(|edge| edge.external_source_data().map(|data| data.source_id) == Some(e_id))
                .cloned()
                .collect();

            assert!(external_source_id_old_e.is_empty());
        }
    }

    Ok(())
}

#[test]
fn replace_node() -> SplitGraphResult<()> {
    let mut splitgraph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> =
        SplitGraph::new(2);

    let mut nodes: Vec<TestNodeWeight> = ["1", "2", "3", "4", "5", "6"]
        .into_iter()
        .map(|name| TestNodeWeight {
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
            name: name.to_string(),
            merkle_tree_hash: MerkleTreeHash::nil(),
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
        let mut splitgraph = SplitGraph::new(9);
        let mut unsplitgraph = SplitGraph::new(32678);

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
            let lineage_id = SplitGraphNodeId::new();
            splitgraph.add_or_replace_node(TestNodeWeight {
                id,
                lineage_id,
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
            })?;
            unsplitgraph.add_or_replace_node(TestNodeWeight {
                id,
                lineage_id,
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
            })?;
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

            if ordered {
                splitgraph.add_ordered_edge(split_from_id, TestEdgeWeight::EdgeA, to_id)?;
                unsplitgraph.add_ordered_edge(unsplit_from_id, TestEdgeWeight::EdgeA, to_id)?;
            } else {
                splitgraph.add_edge(split_from_id, TestEdgeWeight::EdgeA, to_id)?;
                unsplitgraph.add_edge(unsplit_from_id, TestEdgeWeight::EdgeA, to_id)?;
            }

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

            if !outgoing_targets.is_empty() {
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

            if !incoming_sources.is_empty() {
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
fn detect_changes_no_difference() -> SplitGraphResult<()> {
    let mut base_graph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> =
        SplitGraph::new(200);
    base_graph.cleanup_and_merkle_tree_hash();
    let mut updated_graph = base_graph.clone();
    updated_graph.cleanup_and_merkle_tree_hash();

    assert!(base_graph.detect_changes(&updated_graph)?.is_empty());

    Ok(())
}

#[test]
fn detect_changes_simple() -> SplitGraphResult<()> {
    for split_max in [1, 2, 3, 1000] {
        let mut base_graph = SplitGraph::new(split_max);
        let nodes = [
            ("severian"),
            ("thecla"),
            ("terminus est"),
            ("dorcas"),
            ("vodalus"),
            ("drotte"),
        ];

        // root --> severian --> thecla --> vodalus --> drotte
        //                   --> terminus est
        //                   --> vodalus (--> drotte)

        let edges = [
            (None, TestEdgeWeight::EdgeA, "severian", false),
            (Some("severian"), TestEdgeWeight::EdgeA, "thecla", true),
            (Some("thecla"), TestEdgeWeight::EdgeA, "vodalus", false),
            (
                Some("severian"),
                TestEdgeWeight::EdgeA,
                "terminus est",
                true,
            ),
            (Some("severian"), TestEdgeWeight::EdgeA, "vodalus", true),
            (Some("vodalus"), TestEdgeWeight::EdgeA, "drotte", false),
        ];

        let node_id_map = add_nodes_to_splitgraph(&mut base_graph, &nodes);
        add_edges_to_splitgraph(&mut base_graph, &edges, &node_id_map);
        base_graph.cleanup_and_merkle_tree_hash();

        // Changing severian should update severian, root
        let mut updated_graph = base_graph.clone();

        let severian_id = node_id_map
            .get(&"severian")
            .copied()
            .expect("should get severian's id");

        let mut severian = updated_graph
            .node_weight(severian_id)
            .cloned()
            .expect("severian node should exist");
        severian.name = "severian the torturer".into();
        updated_graph.add_or_replace_node(severian)?;
        updated_graph.cleanup_and_merkle_tree_hash();

        let changes = updated_graph.detect_changes(&base_graph)?;

        let changed_ids: Vec<_> = changes
            .into_iter()
            .map(|change| change.entity_id.into_inner().into())
            .collect();

        assert_eq!(
            &[updated_graph.root_id()?, severian_id],
            changed_ids.as_slice()
        );

        // Changing thecla should update severian, root
        let mut updated_graph = base_graph.clone();

        let thecla_id = node_id_map
            .get(&"thecla")
            .copied()
            .expect("should get thecla's id");

        let mut thecla = updated_graph
            .node_weight(thecla_id)
            .cloned()
            .expect("thecla node should exist");
        thecla.name = "chatelaine thecla".into();
        updated_graph.add_or_replace_node(thecla)?;
        updated_graph.cleanup_and_merkle_tree_hash();

        let changes = updated_graph.detect_changes(&base_graph)?;

        let changed_ids: Vec<_> = changes
            .into_iter()
            .map(|change| change.entity_id.into_inner().into())
            .collect();

        assert_eq!(
            &[updated_graph.root_id()?, severian_id, thecla_id],
            changed_ids.as_slice()
        );

        // Changing vodalus should update vodalus, thecla, severian, root
        let mut updated_graph = base_graph.clone();

        let vodalus_id = node_id_map
            .get(&"vodalus")
            .copied()
            .expect("should get thecla's id");

        let mut vodalus = updated_graph
            .node_weight(vodalus_id)
            .cloned()
            .expect("vodalus node should exist");
        vodalus.name = "vodalus the exultant".into();
        updated_graph.add_or_replace_node(vodalus)?;
        updated_graph.cleanup_and_merkle_tree_hash();

        let changes = updated_graph.detect_changes(&base_graph)?;

        // Order of changes is trickier to predict now
        let changed_ids: HashSet<_> = changes
            .into_iter()
            .map(|change| change.entity_id.into_inner().into())
            .collect();

        assert_eq!(
            HashSet::from([updated_graph.root_id()?, severian_id, thecla_id, vodalus_id]),
            changed_ids,
        );

        // Changing drotte should update the whole graph except terminus est
        let mut updated_graph = base_graph.clone();

        let drotte_id = node_id_map
            .get(&"drotte")
            .copied()
            .expect("should get drotte's id");

        let mut drotte = updated_graph
            .node_weight(drotte_id)
            .cloned()
            .expect("drotte node should exist");
        drotte.name = "drotte the journeyman".into();
        updated_graph.add_or_replace_node(drotte)?;
        updated_graph.cleanup_and_merkle_tree_hash();

        let changes = updated_graph.detect_changes(&base_graph)?;

        let changed_ids: HashSet<_> = changes
            .into_iter()
            .map(|change| change.entity_id.into_inner().into())
            .collect();

        assert_eq!(
            HashSet::from([
                updated_graph.root_id()?,
                severian_id,
                thecla_id,
                vodalus_id,
                drotte_id
            ]),
            changed_ids,
        );
    }

    Ok(())
}

#[test]
fn detect_and_perform_updates_ordered_containers() -> SplitGraphResult<()> {
    for split_max in [1, 2, 1000] {
        let mut base_graph = SplitGraph::new(split_max);
        base_graph.cleanup_and_merkle_tree_hash();
        let mut updated_graph = base_graph.clone();
        updated_graph.cleanup_and_merkle_tree_hash();

        let damaya = TestNodeWeight {
            name: "damaya".to_string(),
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
            merkle_tree_hash: MerkleTreeHash::nil(),
        };

        let evil_earth = TestNodeWeight {
            name: "evil_earth".to_string(),
            id: Ulid::new(),
            lineage_id: SplitGraphNodeId::new(),
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
                lineage_id: SplitGraphNodeId::new(),
                merkle_tree_hash: MerkleTreeHash::nil(),
            };
            ordered_child_ids.push(new_node.id());
            updated_graph.add_or_replace_node(new_node.clone())?;
            name_to_id_map.insert(name.to_string(), new_node.id());
            updated_graph.add_ordered_edge(
                damaya.id(),
                TestEdgeWeight::EdgeB { is_default: false },
                new_node.id(),
            )?;
            updated_graph.add_edge(evil_earth.id(), TestEdgeWeight::EdgeA, new_node.id())?;
        }

        updated_graph.cleanup_and_merkle_tree_hash();
        let updates = base_graph.detect_updates(&updated_graph);

        assert!(!updates.is_empty());

        base_graph.perform_updates(&updates);
        base_graph.cleanup_and_merkle_tree_hash();

        let updates = base_graph.detect_updates(&updated_graph);
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

        assert!(
            evil_earth_outgoing_base
                .difference(&evil_earth_outgoing_updated)
                .next()
                .is_none()
        );

        updated_graph.reorder_node(damaya.id(), |order| order.iter().copied().rev().collect())?;

        let subgraph_for_damaya = updated_graph.subgraph_index_for_node(damaya.id()).unwrap();
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

        let updates_after_reorder = base_graph.detect_updates(&updated_graph);
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
    let mut base_graph = SplitGraph::new(3200);
    base_graph.cleanup_and_merkle_tree_hash();
    let mut updated_graph = base_graph.clone();
    updated_graph.cleanup_and_merkle_tree_hash();

    assert!(base_graph.detect_updates(&updated_graph).is_empty());

    let new_node = TestNodeWeight {
        name: "damaya".to_string(),
        id: Ulid::new(),
        lineage_id: SplitGraphNodeId::new(),
        merkle_tree_hash: MerkleTreeHash::nil(),
    };

    updated_graph.add_or_replace_node(new_node.clone())?;
    updated_graph.add_edge(
        updated_graph.root_id()?,
        TestEdgeWeight::EdgeA,
        new_node.id(),
    )?;
    updated_graph.cleanup_and_merkle_tree_hash();

    let updates = base_graph.detect_updates(&updated_graph);

    assert_eq!(2, updates.len());

    let update_1 = updates.first().unwrap();
    let update_2 = updates.get(1).unwrap();

    assert!(matches!(
        update_1,
        Update::NewNode {
            node_weight: SplitGraphNodeWeight::Custom(TestNodeWeight { .. }),
            ..
        }
    ));

    let Update::NewNode {
        node_weight: SplitGraphNodeWeight::Custom(custom_node),
        ..
    } = update_1
    else {
        unreachable!("we already asserted this!")
    };

    assert_eq!(new_node.node_hash(), custom_node.node_hash());

    assert!(matches!(
        update_2,
        Update::NewEdge {
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

    assert_eq!(updated_graph.root_id()?, source.id);
    assert_eq!(new_node.id(), destination.id);

    let inverse_updates = updated_graph.detect_updates(&base_graph);
    assert_eq!(1, inverse_updates.len());

    assert!(matches!(
        inverse_updates.first().unwrap(),
        Update::RemoveEdge { .. }
    ));

    let mut second_updated_graph = updated_graph.clone();
    let mut updated_node = new_node.clone();
    updated_node.set_name("syenite".into());
    second_updated_graph.add_or_replace_node(updated_node)?;
    second_updated_graph.cleanup_and_merkle_tree_hash();
    let replace_node_update = updated_graph.detect_updates(&second_updated_graph);
    assert!(matches!(
        replace_node_update.first().unwrap(),
        Update::ReplaceNode {
            node_weight: SplitGraphNodeWeight::Custom(TestNodeWeight { .. }),
            base_graph_node_id: None,
            ..
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
    let node_id_map = add_nodes_to_subgraph(&mut subgraph, &nodes);

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
        );
    }

    subgraph.remove_externals();
    subgraph.cleanup_maps();

    let subgraph_root_id = subgraph
        .graph
        .node_weight(subgraph.root_index)
        .unwrap()
        .id();

    let expected_edges: Vec<Update<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants>> =
        edges
            .into_iter()
            .map(|(source, target)| {
                let source_id = source
                    .map(|source| node_id_map.get(&source).copied().unwrap())
                    .unwrap_or(subgraph_root_id);
                let destination_id = node_id_map.get(&target).copied().unwrap();

                let source = subgraph.node_weight(source_id).unwrap();
                let destination = subgraph.node_weight(destination_id).unwrap();

                Update::NewEdge {
                    source: source.into(),
                    destination: destination.into(),
                    edge_weight: SplitGraphEdgeWeight::Custom(TestEdgeWeight::EdgeA),
                    subgraph_root_id,
                }
            })
            .collect();

    let updates = subgraph_as_updates(&subgraph, subgraph_root_id);
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

#[test]
fn graph_dfs() -> SplitGraphResult<()> {
    let mut split_graph = SplitGraph::new(1);

    let nodes = &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"];
    let node_id_map = add_nodes_to_splitgraph(
        &mut split_graph,
        &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"],
    );

    let mut first_layer = vec![];
    for node in nodes.iter().take(3).copied() {
        let node_id = node_id_map.get(node).copied().unwrap();
        first_layer.push(node_id);
        split_graph.add_edge(split_graph.root_id()?, TestEdgeWeight::EdgeA, node_id)?;
    }

    let mut second_layer = vec![];
    for first_layer_node_id in first_layer {
        for node in nodes.iter().skip(3).take(3).copied() {
            let node_id = node_id_map.get(node).copied().unwrap();
            second_layer.push(node_id);
            split_graph.add_edge(first_layer_node_id, TestEdgeWeight::EdgeA, node_id)?;
        }
    }

    for second_layer_node_id in second_layer {
        for node in nodes.iter().skip(6).take(6).copied() {
            let node_id = node_id_map.get(node).copied().unwrap();
            split_graph.add_edge(second_layer_node_id, TestEdgeWeight::EdgeA, node_id)?;
        }
    }

    let mut expected_nodes = HashSet::from_iter(node_id_map.values().copied());
    expected_nodes.insert(split_graph.root_id()?);

    let mut dfs = petgraph::visit::DfsPostOrder::new(&split_graph, split_graph.root_id()?);
    let mut found_nodes = HashSet::new();
    while let Some(node_id) = dfs.next(&split_graph) {
        found_nodes.insert(node_id);
    }

    assert_eq!(expected_nodes, found_nodes);

    petgraph::visit::Dfs::new(&split_graph, split_graph.root_id()?);
    let mut dfs = petgraph::visit::DfsPostOrder::new(&split_graph, split_graph.root_id()?);
    let mut found_nodes = HashSet::new();
    while let Some(node_id) = dfs.next(&split_graph) {
        found_nodes.insert(node_id);
    }

    assert_eq!(expected_nodes, found_nodes);

    let mut found_nodes = HashSet::new();
    petgraph::visit::depth_first_search(&split_graph, Some(split_graph.root_id()?), |event| {
        match event {
            DfsEvent::Discover(node_id, _) => {
                found_nodes.insert(node_id);
            }
            DfsEvent::BackEdge(_, _) => {
                panic!("should not have back edges (they indicate a cycle)");
            }
            _ => {}
        }

        Control::<()>::Continue
    });

    assert_eq!(expected_nodes, found_nodes);

    Ok(())
}

#[test]
fn graph_cycle_test() -> SplitGraphResult<()> {
    let mut split_graph = SplitGraph::new(3);

    let mut node_id_map =
        add_nodes_to_splitgraph(&mut split_graph, &["a", "b", "c", "d", "e", "f"]);
    for node_id in node_id_map.values().copied() {
        split_graph.add_edge_with_cycle_check(
            split_graph.root_id()?,
            TestEdgeWeight::EdgeA,
            node_id,
        )?;
    }

    split_graph.add_edge_with_cycle_check(
        node_id_map.get(&"a").copied().unwrap(),
        TestEdgeWeight::EdgeA,
        node_id_map.get(&"b").copied().unwrap(),
    )?;

    split_graph.add_edge_with_cycle_check(
        node_id_map.get(&"b").copied().unwrap(),
        TestEdgeWeight::EdgeA,
        node_id_map.get(&"c").copied().unwrap(),
    )?;

    split_graph.add_edge_with_cycle_check(
        node_id_map.get(&"a").copied().unwrap(),
        TestEdgeWeight::EdgeA,
        node_id_map.get(&"c").copied().unwrap(),
    )?;

    split_graph.add_edge_with_cycle_check(
        node_id_map.get(&"a").copied().unwrap(),
        TestEdgeWeight::EdgeB { is_default: false },
        node_id_map.get(&"c").copied().unwrap(),
    )?;

    assert!(
        split_graph
            .add_edge_with_cycle_check(
                node_id_map.get(&"c").copied().unwrap(),
                TestEdgeWeight::EdgeA,
                node_id_map.get(&"a").copied().unwrap(),
            )
            .is_err()
    );

    assert!(split_graph.is_acyclic_directed());

    split_graph.add_edge_with_cycle_check(
        node_id_map.get(&"c").copied().unwrap(),
        TestEdgeWeight::EdgeA,
        node_id_map.get(&"f").copied().unwrap(),
    )?;

    assert!(
        split_graph
            .add_edge_with_cycle_check(
                node_id_map.get(&"f").copied().unwrap(),
                TestEdgeWeight::EdgeA,
                node_id_map.get(&"a").copied().unwrap(),
            )
            .is_err()
    );

    assert!(split_graph.is_acyclic_directed());

    node_id_map.extend(add_nodes_to_splitgraph(
        &mut split_graph,
        &["g", "h", "i", "j", "k", "l"],
    ));

    for edge in [
        TestEdgeWeight::EdgeA,
        TestEdgeWeight::EdgeB { is_default: false },
    ] {
        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"a").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"g").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"g").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"h").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"h").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"i").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"h").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"c").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"h").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"b").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"h").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"j").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"h").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"k").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"h").copied().unwrap(),
            edge.clone(),
            node_id_map.get(&"l").copied().unwrap(),
        )?;

        split_graph.add_edge_with_cycle_check(
            node_id_map.get(&"l").copied().unwrap(),
            edge.clone(),
            node_id_map.get("b").copied().unwrap(),
        )?;
    }

    assert!(
        split_graph
            .add_edge_with_cycle_check(
                node_id_map.get(&"l").copied().unwrap(),
                TestEdgeWeight::EdgeA,
                node_id_map.get("a").copied().unwrap(),
            )
            .is_err()
    );

    Ok(())
}

#[test]
fn graph_cycle_test_mimic_component_parentage() -> SplitGraphResult<()> {
    for split_max in [1, 2, 3, 500] {
        let mut split_graph = SplitGraph::new(split_max);

        let node_id_map = add_nodes_to_splitgraph(
            &mut split_graph,
            &[
                "components",
                "variants",
                "sv1",
                "sv1_prop",
                "sv1_prop2",
                "sv1_prop3",
                "sv1_prop4",
                "sv1_prop5",
                "sv1_prop6",
                "sv2",
                "sv2_prop",
                "sv2_prop2",
                "sv2_prop3",
                "component1",
                "component1_av",
                "component1_av2",
                "component1_av3",
                "component1_av4",
                "component1_av5",
                "component1_av6",
                "component2",
                "component2_av",
                "component2_av2",
                "component2_av3",
            ],
        );

        let edges = [
            (None, "components"),
            (None, "variants"),
            (Some("variants"), "sv1"),
            (Some("sv1"), "sv1_prop"),
            (Some("sv1_prop"), "sv1_prop2"),
            (Some("sv1_prop2"), "sv1_prop3"),
            (Some("sv1_prop2"), "sv1_prop4"),
            (Some("sv1_prop4"), "sv1_prop5"),
            (Some("sv1_prop4"), "sv1_prop6"),
            (Some("components"), "component1"),
            (Some("component1"), "sv1"),
            (Some("component1"), "component1_av"),
            (Some("component1_av"), "sv1_prop"),
            (Some("component1_av"), "component1_av2"),
            (Some("component1_av2"), "sv1_prop2"),
            (Some("component1_av2"), "component1_av3"),
            (Some("component1_av3"), "sv1_prop3"),
            (Some("component1_av2"), "component1_av4"),
            (Some("component1_av4"), "sv1_prop4"),
            (Some("component1_av4"), "component1_av5"),
            (Some("component1_av5"), "sv1_prop5"),
            (Some("component1_av5"), "component1_av6"),
            (Some("component1_av6"), "sv1_prop6"),
            (Some("variants"), "sv2"),
            (Some("sv2"), "sv2_prop"),
            (Some("components"), "component2"),
            (Some("component2"), "component2_av"),
            (Some("component2"), "sv2"),
            (Some("component2_av"), "sv2_prop"),
            (Some("component2_av"), "component2_av2"),
            (Some("component2_av2"), "sv2_prop2"),
            (Some("component2_av2"), "component2_av3"),
            (Some("component2_av3"), "sv2_prop3"),
        ];

        for (from, to) in edges {
            let from_id = match from {
                Some(from) => node_id_map.get(from).copied().unwrap(),
                None => split_graph.root_id()?,
            };
            let to_id = node_id_map.get(to).copied().unwrap();

            split_graph.add_edge_with_cycle_check(from_id, TestEdgeWeight::EdgeA, to_id)?;
        }

        split_graph.add_edge_with_cycle_check(
            node_id_map.get("component1").copied().unwrap(),
            TestEdgeWeight::EdgeB { is_default: true },
            node_id_map.get("component2").copied().unwrap(),
        )?;

        assert!(
            split_graph
                .add_edge_with_cycle_check(
                    node_id_map.get("component2").copied().unwrap(),
                    TestEdgeWeight::EdgeB { is_default: true },
                    node_id_map.get("component1").copied().unwrap(),
                )
                .is_err()
        );
    }

    Ok(())
}

#[test]
fn into_node_identifiers() -> SplitGraphResult<()> {
    let mut split_graph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> =
        SplitGraph::new(2);

    let node_id_map = add_nodes_to_splitgraph(&mut split_graph, &["a", "b", "c", "d", "e", "f"]);

    let ids: Vec<_> = split_graph.node_identifiers().collect();
    // 3 graph roots, 6 test nodes
    assert_eq!(9, ids.len());

    let mut custom_nodes = Vec::new();
    let mut graph_roots = Vec::new();
    for id in ids {
        let weight = split_graph.raw_node_weight(id);
        assert!(weight.is_some());
        match weight {
            Some(weight) => match weight {
                SplitGraphNodeWeight::Custom(_) => {
                    custom_nodes.push(id);
                }
                SplitGraphNodeWeight::ExternalTarget { .. } => {
                    panic!("there should be no external targets");
                }
                SplitGraphNodeWeight::Ordering { .. } => {
                    panic!("there should be no ordering nodes");
                }
                SplitGraphNodeWeight::GraphRoot { id, .. }
                | SplitGraphNodeWeight::SubGraphRoot { id, .. } => {
                    graph_roots.push(id);
                }
            },
            None => unreachable!("i just asserted it was some"),
        }
    }

    assert_eq!(6, custom_nodes.len());
    assert_eq!(3, graph_roots.len());

    let node_ids: HashSet<SplitGraphNodeId> = HashSet::from_iter(node_id_map.into_values());
    let custom_nodes = HashSet::from_iter(custom_nodes);

    assert_eq!(node_ids, custom_nodes);

    Ok(())
}

#[test]
fn into_neighbors() -> SplitGraphResult<()> {
    let mut split_graph: SplitGraph<TestNodeWeight, TestEdgeWeight, TestEdgeWeightDiscriminants> =
        SplitGraph::new(2);

    let node_id_map = add_nodes_to_splitgraph(&mut split_graph, &["a", "b", "c", "d", "e", "f"]);
    for &node_id in node_id_map.values() {
        split_graph.add_edge(split_graph.root_id()?, TestEdgeWeight::EdgeA, node_id)?;
    }

    for id in split_graph.node_identifiers() {
        dbg!(split_graph.raw_node_weight(id));
        for neighbor_id in split_graph.neighbors(id) {
            dbg!(split_graph.raw_node_weight(neighbor_id));
        }
    }

    Ok(())
}
