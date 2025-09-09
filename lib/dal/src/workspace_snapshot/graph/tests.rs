use std::collections::HashMap;

use si_events::{
    ContentHash,
    ulid::Ulid,
};

use crate::{
    EdgeWeight,
    EdgeWeightKind,
    PropKind,
    workspace_snapshot::{
        graph::WorkspaceSnapshotGraphVCurrent,
        node_weight::NodeWeight,
    },
};

mod detect_changes;
mod detect_updates;
mod exclusive_outgoing_edges;
mod rebase;

#[allow(dead_code)]
fn add_prop_nodes_to_graph<'a, 'b>(
    graph: &'a mut WorkspaceSnapshotGraphVCurrent,
    nodes: &'a [&'b str],
    ordered: bool,
) -> HashMap<&'b str, Ulid> {
    let mut node_id_map = HashMap::new();
    for node in nodes {
        // "props" here are just nodes that are easy to create and render the name on the dot
        // output. there is no domain modeling in this test.
        let node_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let node_lineage_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let prop_node_weight = NodeWeight::new_prop(
            node_id,
            node_lineage_id,
            PropKind::Object,
            node,
            ContentHash::new(node.as_bytes()),
        );
        if ordered {
            graph
                .add_ordered_node(prop_node_weight)
                .expect("Unable to add prop");
        } else {
            graph
                .add_or_replace_node(prop_node_weight)
                .expect("Unable to add prop");
        }

        node_id_map.insert(*node, node_id);
    }
    node_id_map
}

#[allow(dead_code)]
fn add_edges(
    graph: &mut WorkspaceSnapshotGraphVCurrent,
    node_id_map: &HashMap<&str, Ulid>,
    edges: &[(Option<&str>, &str)],
) {
    for (source, target) in edges {
        let source = match source {
            None => graph.root(),
            Some(node) => graph
                .get_node_index_by_id(
                    node_id_map
                        .get(node)
                        .copied()
                        .expect("source node should have an id"),
                )
                .expect("get node index by id"),
        };

        let target = graph
            .get_node_index_by_id(
                node_id_map
                    .get(target)
                    .copied()
                    .expect("target node should have an id"),
            )
            .expect("get node index by id");

        graph
            .add_edge(source, EdgeWeight::new(EdgeWeightKind::new_use()), target)
            .expect("add edge");
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod test {
    use std::{
        collections::HashSet,
        str::FromStr,
    };

    use petgraph::{
        Outgoing,
        graph::NodeIndex,
        visit::EdgeRef,
    };
    use pretty_assertions_sorted::assert_eq;
    use si_events::{
        ContentHash,
        merkle_tree_hash::MerkleTreeHash,
        ulid::Ulid,
    };

    use super::{
        add_edges,
        add_prop_nodes_to_graph,
    };
    use crate::{
        ComponentId,
        FuncId,
        PropId,
        SchemaId,
        SchemaVariantId,
        workspace_snapshot::{
            content_address::ContentAddress,
            edge_weight::{
                EdgeWeight,
                EdgeWeightKind,
                EdgeWeightKindDiscriminants,
            },
            graph::{
                WorkspaceSnapshotGraphVCurrent,
                detector::Update,
            },
            node_weight::NodeWeight,
        },
    };

    #[test]
    fn new() {
        let graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");
        assert!(graph.is_acyclic_directed());
    }

    // Previously, WorkspaceSnapshotGraph::new would not populate its node_index_by_id, so this test
    // would fail, in addition to any functionality that depended on getting the root node index
    // on a fresh graph (like add_ordered_node)
    #[test]
    fn get_root_index_by_root_id_on_fresh_graph() {
        let graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let root_id = graph
            .get_node_weight(graph.root())
            .expect("get root weight")
            .id();

        let root_node_idx = graph
            .get_node_index_by_id(root_id)
            .expect("get root node index from ULID");

        assert_eq!(graph.root(), root_node_idx);
    }

    #[test]
    fn multiply_parented_nodes() {
        // All edges are outgoing from top to bottom except e to u
        //
        //          root node---->t--->u--->v
        //              |              ^
        //              |              |
        //              r ------       |
        //             / \     |       |
        //            a   b    |       |
        //             \ / \   |       |
        //              c  |   |       |
        //            / |  |   |       |
        //            | d <-   |       |
        //            | |      |       |
        //            ->e<------       |
        //              |              |
        //              ----------------
        //
        // Edge from e to u mimics a function edge from a prop through a prototype to a function
        // There are a few other edges to "u" that are not represented in the drawing above.
        //

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

        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let node_id_map = add_prop_nodes_to_graph(&mut graph, &nodes, false);
        add_edges(&mut graph, &node_id_map, &edges);

        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");

        for (source, target) in edges {
            let source_idx = match source {
                None => graph.root(),
                Some(node) => graph
                    .get_node_index_by_id(
                        node_id_map
                            .get(node)
                            .copied()
                            .expect("source node should have an id"),
                    )
                    .expect("get node index by id"),
            };

            let target_idx = graph
                .get_node_index_by_id(
                    node_id_map
                        .get(target)
                        .copied()
                        .expect("target node should have an id"),
                )
                .expect("get node index by id");

            assert!(
                graph
                    .edges_directed(source_idx, Outgoing)
                    .any(|edge_ref| edge_ref.target() == target_idx),
                "An edge from {} to {} should exist",
                source.unwrap_or("root"),
                target
            );
        }
        assert!(graph.is_acyclic_directed());

        for (_, id) in node_id_map.iter() {
            let idx_for_node = graph
                .get_node_index_by_id(*id)
                .expect("able to get idx by id");
            graph
                .get_node_weight(idx_for_node)
                .expect("node with weight in graph");
        }
    }

    #[test]
    fn add_nodes_and_edges() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");
        let component_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                component_id,
                Ulid::new(),
                ContentAddress::Component(ContentHash::new(
                    ComponentId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add component");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let func_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                func_id,
                Ulid::new(),
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add func");
        let prop_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                prop_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add prop");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                func_index,
            )
            .expect("Unable to add root -> func edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");

        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn cyclic_failure() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let initial_schema_node_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let initial_schema_variant_node_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");
        let component_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let initial_component_node_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                component_id,
                Ulid::new(),
                ContentAddress::Component(ContentHash::new(
                    ComponentId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add component");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                initial_component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                initial_schema_node_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                initial_schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to find NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let pre_cycle_root_index = graph.root();

        // This should cause a cycle.
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to find NodeIndex"),
            )
            .expect_err("Created a cycle");

        assert_eq!(pre_cycle_root_index, graph.root(),);
    }

    #[test]
    fn update_content() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::from("Constellation")),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new("Freestar Collective".as_bytes())),
            ))
            .expect("Unable to add schema variant");
        let component_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                component_id,
                Ulid::new(),
                ContentAddress::Component(ContentHash::from("Crimson Fleet")),
            ))
            .expect("Unable to add component");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Ensure that the root node merkle tree hash looks as we expect before the update.
        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        let pre_update_root_node_merkle_tree_hash: MerkleTreeHash =
            MerkleTreeHash::from_str("3206cdbe645cbc3bfde3d64c5712abf9")
                .expect("able to create hash from hex string");
        assert_eq!(
            pre_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root())
                .expect("could not get node weight")
                .merkle_tree_hash(), // actual
        );

        let updated_content_hash = ContentHash::from("new_content");
        graph
            .update_content(component_id, updated_content_hash)
            .expect("Unable to update Component content hash");
        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");

        let post_update_root_node_merkle_tree_hash: MerkleTreeHash =
            MerkleTreeHash::from_str("fcf780cbc0e2efa2a04c30cef2880cfe")
                .expect("able to create hash from hex string");
        assert_eq!(
            post_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root())
                .expect("could not get node weight")
                .merkle_tree_hash(), // actual
        );
        assert_eq!(
            updated_content_hash, // expected
            graph
                .get_node_weight(
                    graph
                        .get_node_index_by_id(component_id)
                        .expect("could not get node index by id")
                )
                .expect("could not get node weight")
                .content_hash(), // actual
        );

        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");

        // Ensure that there are not more nodes than the ones that should be in use, and the
        // initial ones (categories and default views)
        assert_eq!(17, graph.node_count());

        // The hashes must not change upon cleanup.
        assert_eq!(
            post_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root())
                .expect("could not get node weight")
                .merkle_tree_hash(), // actual
        );
        assert_eq!(
            updated_content_hash, // expected
            graph
                .get_node_weight(
                    graph
                        .get_node_index_by_id(component_id)
                        .expect("could not get node index by id")
                )
                .expect("could not get node weight")
                .content_hash(), // actual
        );
    }

    #[test]
    fn add_ordered_node() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                func_id,
                Ulid::new(),
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add func");
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let prop_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(NodeWeight::new_content(
                prop_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        graph.dot();

        let ordered_prop_1_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_1_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_2_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_3_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");
        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        graph.dot();

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_2_index,
                ordered_prop_3_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
    }

    #[test]
    fn add_ordered_node_below_root() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let prop_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(NodeWeight::new_content(
                prop_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(prop_id.to_string().as_bytes())),
            ))
            .expect("Unable to add prop");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                prop_index,
            )
            .expect("Unable to add root -> prop edge");

        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        assert_eq!(
            Vec::<NodeIndex>::new(),
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
    }

    #[test]
    fn reorder_ordered_node() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                func_id,
                Ulid::new(),
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add func");
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let prop_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(NodeWeight::new_content(
                prop_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        graph.dot();

        let ordered_prop_1_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_1_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_2_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_3_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");

        let ordered_prop_4_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_4_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_4_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_4_index,
            )
            .expect("Unable to add prop -> ordered_prop_4 edge");

        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        graph.dot();

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_2_index,
                ordered_prop_3_index,
                ordered_prop_4_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );

        let new_order = vec![
            ordered_prop_2_id,
            ordered_prop_1_id,
            ordered_prop_4_id,
            ordered_prop_3_id,
        ];

        graph
            .update_order(prop_id, new_order)
            .expect("Unable to update order of prop's children");

        assert_eq!(
            vec![
                ordered_prop_2_index,
                ordered_prop_1_index,
                ordered_prop_4_index,
                ordered_prop_3_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
    }

    #[test]
    fn remove_unordered_node_and_detect_edge_removal() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let schema_variant_2_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_2_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_2_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_2_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let expected_edges = HashSet::from([schema_variant_2_index, schema_variant_index]);

        let existing_edges: HashSet<NodeIndex> = graph
            .edges_directed(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex for schema"),
                Outgoing,
            )
            .map(|edge_ref| edge_ref.target())
            .collect();

        assert_eq!(
            expected_edges, existing_edges,
            "confirm edges are there before deleting"
        );

        let mut graph_with_deleted_edge = graph.clone();

        graph_with_deleted_edge.dot();

        graph_with_deleted_edge
            .remove_edge(
                graph_with_deleted_edge
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex for schema"),
                schema_variant_2_index,
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Edge removal failed");

        graph_with_deleted_edge.dot();

        let existing_edges: Vec<NodeIndex> = graph_with_deleted_edge
            .edges_directed(
                graph_with_deleted_edge
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex for schema"),
                Outgoing,
            )
            .map(|edge_ref| edge_ref.target())
            .collect();

        assert_eq!(
            vec![schema_variant_index],
            existing_edges,
            "confirm edges after deletion"
        );

        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        let updates = graph.detect_updates(&graph_with_deleted_edge);

        assert_eq!(1, updates.len());

        assert!(matches!(
            updates.first().expect("should be there"),
            Update::RemoveEdge { .. }
        ));
    }

    #[test]
    fn remove_unordered_node() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                Ulid::new(),
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let schema_variant_2_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_2_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_2_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_2_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let expected_edges = HashSet::from([schema_variant_2_index, schema_variant_index]);

        let existing_edges: HashSet<NodeIndex> = graph
            .edges_directed(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex for schema"),
                Outgoing,
            )
            .map(|edge_ref| edge_ref.target())
            .collect();

        assert_eq!(
            expected_edges, existing_edges,
            "confirm edges are there before deleting"
        );

        graph
            .remove_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex for schema"),
                schema_variant_2_index,
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Edge removal failed");

        let existing_edges: Vec<NodeIndex> = graph
            .edges_directed(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex for schema"),
                Outgoing,
            )
            .map(|edge_ref| edge_ref.target())
            .collect();

        assert_eq!(
            vec![schema_variant_index],
            existing_edges,
            "confirm edges after deletion"
        );
    }

    #[test]
    fn remove_ordered_node() {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_id,
                Ulid::new(),
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema");
        let schema_variant_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                schema_variant_id,
                Ulid::new(),
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            ))
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                func_id,
                Ulid::new(),
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add func");
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let root_prop_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_index = graph
            .add_ordered_node(NodeWeight::new_content(
                root_prop_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            ))
            .expect("Unable to add prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                root_prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        graph.dot();

        let ordered_prop_1_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_1_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_2_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_3_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");

        let ordered_prop_4_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_4_index = graph
            .add_or_replace_node(NodeWeight::new_content(
                ordered_prop_4_id,
                Ulid::new(),
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            ))
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                ordered_prop_4_index,
            )
            .expect("Unable to add prop -> ordered_prop_4 edge");

        graph
            .cleanup_and_merkle_tree_hash()
            .expect("cleanup and merkle");
        graph.dot();

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_2_index,
                ordered_prop_3_index,
                ordered_prop_4_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(root_prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );

        graph
            .remove_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex for prop"),
                ordered_prop_2_index,
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to remove prop -> ordered_prop_2 edge");

        assert_eq!(
            vec![
                ordered_prop_1_index,
                ordered_prop_3_index,
                ordered_prop_4_index,
            ],
            graph
                .ordered_children_for_node(
                    graph
                        .get_node_index_by_id(root_prop_id)
                        .expect("Unable to get prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
        if let NodeWeight::Ordering(ordering_weight) = graph
            .get_node_weight(
                graph
                    .ordering_node_index_for_container(
                        graph
                            .get_node_index_by_id(root_prop_id)
                            .expect("Unable to find ordering node for prop"),
                    )
                    .expect("Error getting ordering NodeIndex for prop")
                    .expect("Unable to find ordering NodeIndex"),
            )
            .expect("Unable to get ordering NodeWeight for ordering node")
        {
            assert_eq!(
                &vec![ordered_prop_1_id, ordered_prop_3_id, ordered_prop_4_id],
                ordering_weight.order()
            );
        } else {
            panic!("Unable to destructure ordering node weight");
        }
    }
}
