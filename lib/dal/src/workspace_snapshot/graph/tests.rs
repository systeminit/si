mod rebase;

#[allow(clippy::panic)]
#[cfg(test)]
mod test {
    use petgraph::graph::NodeIndex;
    use petgraph::visit::EdgeRef;
    use petgraph::Outgoing;
    use pretty_assertions_sorted::assert_eq;
    use si_events::merkle_tree_hash::MerkleTreeHash;
    use si_events::ContentHash;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;

    use crate::change_set::ChangeSet;
    use crate::workspace_snapshot::conflict::Conflict;
    use crate::workspace_snapshot::content_address::ContentAddress;
    use crate::workspace_snapshot::edge_weight::{
        EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
    };
    use crate::workspace_snapshot::node_weight::NodeWeight;
    use crate::workspace_snapshot::update::Update;
    use crate::WorkspaceSnapshotGraph;
    use crate::{ComponentId, FuncId, PropId, PropKind, SchemaId, SchemaVariantId};

    #[derive(Debug, PartialEq)]
    struct ConflictsAndUpdates {
        conflicts: Vec<Conflict>,
        updates: Vec<Update>,
    }

    #[test]
    fn new() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        assert!(graph.is_acyclic_directed());
    }

    // Previously, WorkspaceSnapshotGraph::new would not populate its node_index_by_id, so this test
    // would fail, in addition to any functionality that depended on getting the root node index
    // on a fresh graph (like add_ordered_node)
    #[test]
    fn get_root_index_by_root_id_on_fresh_graph() {
        let base_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let active_change_set = &base_change_set;
        let graph = WorkspaceSnapshotGraph::new(active_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let root_id = graph
            .get_node_weight(graph.root_index)
            .expect("get root weight")
            .id();

        let root_node_idx = graph
            .get_node_index_by_id(root_id)
            .expect("get root node index from ULID");

        assert_eq!(graph.root_index, root_node_idx);
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

        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let mut node_id_map = HashMap::new();

        for node in nodes {
            // "props" here are just nodes that are easy to create and render the name on the dot
            // output. there is no domain modeling in this test.
            let node_id = change_set.generate_ulid().expect("Unable to generate Ulid");
            let prop_node_weight = NodeWeight::new_prop(
                change_set,
                node_id,
                PropKind::Object,
                node,
                ContentHash::new(node.as_bytes()),
            )
            .expect("create prop node weight");
            graph
                .add_node(prop_node_weight)
                .expect("Unable to add prop");

            node_id_map.insert(node, node_id);
        }

        for (source, target) in edges {
            let source = match source {
                None => graph.root_index,
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
                .add_edge(
                    source,
                    EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                        .expect("create edge weight"),
                    target,
                )
                .expect("add edge");
        }

        graph.cleanup();

        for (source, target) in edges {
            let source_idx = match source {
                None => graph.root_index,
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
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    func_id,
                    ContentAddress::Func(ContentHash::new(
                        FuncId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add func");
        let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        PropId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add prop");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");

        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn cyclic_failure() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let initial_schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let initial_schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let initial_component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::new(
                        ComponentId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                initial_component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                initial_schema_node_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                initial_schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to find NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let pre_cycle_root_index = graph.root_index;

        // This should cause a cycle.
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to find NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to find NodeIndex"),
            )
            .expect_err("Created a cycle");

        assert_eq!(pre_cycle_root_index, graph.root_index,);
    }

    #[test]
    fn update_content() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Constellation")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        "Freestar Collective".as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Crimson Fleet")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Ensure that the root node merkle tree hash looks as we expect before the update.
        let pre_update_root_node_merkle_tree_hash: MerkleTreeHash =
            MerkleTreeHash::from_str("49a6baef5d1c29f43653e0b7c02dfb73")
                .expect("able to create hash from hex string");
        assert_eq!(
            pre_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root_index)
                .expect("could not get node weight")
                .merkle_tree_hash(), // actual
        );

        let updated_content_hash = ContentHash::from("new_content");
        graph
            .update_content(change_set, component_id, updated_content_hash)
            .expect("Unable to update Component content hash");

        let post_update_root_node_merkle_tree_hash: MerkleTreeHash =
            MerkleTreeHash::from_str("75febafba241026c63e27ab5b129cb26")
                .expect("able to create hash from hex string");
        assert_eq!(
            post_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root_index)
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

        graph.cleanup();

        // Ensure that there are not more nodes than the ones that should be in use.
        assert_eq!(4, graph.node_count());

        // The hashes must not change upon cleanup.
        assert_eq!(
            post_update_root_node_merkle_tree_hash, // expected
            graph
                .get_node_weight(graph.root_index)
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
    fn detect_conflicts_and_updates_simple_no_conflicts_no_updates_in_base() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        initial_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        let component_id = new_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_change_set,
                    component_id,
                    ContentAddress::Schema(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        new_graph
            .add_edge(
                new_graph.root_index,
                EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &initial_graph,
                initial_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_purely_new_content_in_base() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let new_graph = base_graph.clone();

        let new_onto_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let new_onto_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    new_onto_component_id,
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        let _new_onto_root_component_edge_index = base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &base_graph,
                base_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);

        let new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(new_graph.root_index, *source);
                assert_eq!(new_onto_component_index, *destination);
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_with_purely_new_content_in_new_graph() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        let new_component_id = new_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let new_component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_change_set,
                    new_component_id,
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        new_graph
            .add_edge(
                new_graph.root_index,
                EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_component_index,
            )
            .expect("Unable to add root -> component edge");

        new_graph.cleanup();
        println!("Updated new graph (Root: {:?}):", new_graph.root_index);
        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &base_graph,
                base_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert!(updates.is_empty());
        assert!(conflicts.is_empty());

        let (conflicts, updates) = base_graph
            .detect_conflicts_and_updates(
                base_change_set.vector_clock_id(),
                &new_graph,
                new_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts.is_empty());

        match updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(base_graph.root_index, *source);
                assert_eq!(new_component_index, *destination);
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_updates_on_both_sides() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        let component_id = new_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        new_graph
            .add_edge(
                new_graph.root_index,
                EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!("new graph (Root {:?}):", new_graph.root_index);
        new_graph.dot();

        let new_onto_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let new_onto_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    new_onto_component_id,
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &base_graph,
                base_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);

        let new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(new_graph.root_index, *source);
                assert_eq!(new_onto_component_index, *destination);
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_content_conflict() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        new_graph
            .update_content(
                new_change_set,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
        println!("new graph (Root {:?}):", new_graph.root_index);
        new_graph.dot();

        base_graph
            .update_content(
                base_change_set,
                component_id,
                ContentHash::from("Base Updated Component A"),
            )
            .expect("Unable to update Component A");

        base_graph.cleanup();
        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &base_graph,
                base_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::NodeContent {
                onto: base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get component NodeIndex"),
                to_rebase: new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get component NodeIndex"),
            }],
            conflicts
        );
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_modify_removed_item_conflict() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    component_id,
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        base_graph
            .remove_edge(
                base_change_set,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to remove Component A");

        base_graph.cleanup();
        println!("Updated base graph (Root: {:?}):", base_graph.root_index);
        base_graph.dot();

        new_graph
            .update_content(
                new_change_set,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
        println!("new graph (Root {:?}):", new_graph.root_index);
        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &base_graph,
                base_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::ModifyRemovedItem(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex")
            )],
            conflicts
        );
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_add_unordered_child_to_ordered_container() {
        let base_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let active_change_set = &base_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(active_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let active_graph = &mut base_graph;

        // Create base prop node
        let base_prop_id = {
            let prop_id = active_change_set
                .generate_ulid()
                .expect("Unable to generate Ulid");
            let prop_index = active_graph
                .add_ordered_node(
                    active_change_set,
                    NodeWeight::new_content(
                        active_change_set,
                        prop_id,
                        ContentAddress::Prop(ContentHash::new(prop_id.to_string().as_bytes())),
                    )
                    .expect("Unable to create NodeWeight"),
                )
                .expect("Unable to add prop");

            active_graph
                .add_edge(
                    active_graph.root_index,
                    EdgeWeight::new(active_change_set, EdgeWeightKind::new_use())
                        .expect("Unable to create EdgeWeight"),
                    prop_index,
                )
                .expect("Unable to add sv -> prop edge");

            prop_id
        };

        active_graph.cleanup();
        active_graph.dot();

        // Create two prop nodes children of base prop
        let ordered_prop_1_index = {
            let ordered_prop_id = active_change_set
                .generate_ulid()
                .expect("Unable to generate Ulid");
            let ordered_prop_index = active_graph
                .add_node(
                    NodeWeight::new_content(
                        active_change_set,
                        ordered_prop_id,
                        ContentAddress::Prop(ContentHash::new(
                            ordered_prop_id.to_string().as_bytes(),
                        )),
                    )
                    .expect("Unable to create NodeWeight"),
                )
                .expect("Unable to add ordered prop");
            active_graph
                .add_ordered_edge(
                    active_change_set,
                    active_graph
                        .get_node_index_by_id(base_prop_id)
                        .expect("Unable to get prop NodeIndex"),
                    EdgeWeight::new(active_change_set, EdgeWeightKind::new_use())
                        .expect("Unable to create uses edge weight"),
                    ordered_prop_index,
                )
                .expect("Unable to add prop -> ordered_prop_1 edge");

            ordered_prop_index
        };

        active_graph.cleanup();
        active_graph.dot();

        let attribute_prototype_id = {
            let node_id = active_change_set
                .generate_ulid()
                .expect("Unable to generate Ulid");
            let node_index = active_graph
                .add_node(
                    NodeWeight::new_content(
                        active_change_set,
                        node_id,
                        ContentAddress::AttributePrototype(ContentHash::new(
                            node_id.to_string().as_bytes(),
                        )),
                    )
                    .expect("Unable to create NodeWeight"),
                )
                .expect("Unable to add attribute prototype");

            active_graph
                .add_edge(
                    active_graph.root_index,
                    EdgeWeight::new(active_change_set, EdgeWeightKind::new_use())
                        .expect("Unable to create EdgeWeight"),
                    node_index,
                )
                .expect("Unable to add root -> prototype edge");

            node_id
        };

        active_graph.cleanup();
        active_graph.dot();

        // Get new graph
        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let active_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();
        let active_graph = &mut new_graph;

        // Connect Prototype to Prop
        active_graph
            .add_edge(
                active_graph
                    .get_node_index_by_id(base_prop_id)
                    .expect("Unable to get prop NodeIndex"),
                EdgeWeight::new(active_change_set, EdgeWeightKind::Prototype(None))
                    .expect("Unable to create EdgeWeight"),
                active_graph
                    .get_node_index_by_id(attribute_prototype_id)
                    .expect("Unable to get prop NodeIndex"),
            )
            .expect("Unable to add sv -> prop edge");
        active_graph.cleanup();
        active_graph.dot();

        assert_eq!(
            vec![ordered_prop_1_index,],
            new_graph
                .ordered_children_for_node(
                    new_graph
                        .get_node_index_by_id(base_prop_id)
                        .expect("Unable to get base prop NodeIndex")
                )
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );

        // Assert that the new edge to the prototype gets created
        let (conflicts, updates) = base_graph
            .detect_conflicts_and_updates(
                active_change_set.vector_clock_id(),
                &new_graph,
                new_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts.is_empty());

        match updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(
                    base_graph
                        .get_node_index_by_id(base_prop_id)
                        .expect("Unable to get prop NodeIndex"),
                    *source
                );
                assert_eq!(
                    base_graph
                        .get_node_index_by_id(attribute_prototype_id)
                        .expect("Unable to get prop NodeIndex"),
                    *destination
                );
                assert_eq!(&EdgeWeightKind::Prototype(None), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_complex() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &initial_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Docker Image Schema
        let docker_image_schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let docker_image_schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    docker_image_schema_id,
                    ContentAddress::Schema(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        // Docker Image Schema Variant
        let docker_image_schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let docker_image_schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    docker_image_schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(docker_image_schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        // Nginx Docker Image Component
        let nginx_docker_image_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let nginx_docker_image_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    nginx_docker_image_component_id,
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                nginx_docker_image_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(nginx_docker_image_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Alpine Component
        let alpine_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let alpine_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    alpine_component_id,
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                alpine_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(alpine_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        // Butane Schema
        let butane_schema_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let butane_schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    butane_schema_id,
                    ContentAddress::Schema(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                butane_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        // Butane Schema Variant
        let butane_schema_variant_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let butane_schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    butane_schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(butane_schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                butane_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        // Nginx Butane Component
        let nginx_butane_component_id = base_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let nginx_butane_node_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    base_change_set,
                    nginx_butane_component_id,
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                nginx_butane_node_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(butane_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        println!("Initial base graph (Root {:?}):", base_graph.root_index);
        base_graph.dot();

        // Create a new change set to cause some problems!
        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = base_graph.clone();

        // Create a modify removed item conflict.
        base_graph
            .remove_edge(
                base_change_set,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to update the component");
        new_graph
            .update_content(
                new_change_set,
                nginx_butane_component_id,
                ContentHash::from("second"),
            )
            .expect("Unable to update the component");

        // Create a node content conflict.
        base_graph
            .update_content(
                base_change_set,
                docker_image_schema_variant_id,
                ContentHash::from("oopsie"),
            )
            .expect("Unable to update the component");
        new_graph
            .update_content(
                new_change_set,
                docker_image_schema_variant_id,
                ContentHash::from("poopsie"),
            )
            .expect("Unable to update the component");

        // Create a pure update.
        base_graph
            .update_content(
                base_change_set,
                docker_image_schema_id,
                ContentHash::from("bg3"),
            )
            .expect("Unable to update the schema");

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &base_graph,
                base_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        println!("base graph current root: {:?}", base_graph.root_index);
        base_graph.dot();
        println!("new graph current root: {:?}", new_graph.root_index);
        new_graph.dot();

        let expected_conflicts = vec![
            Conflict::ModifyRemovedItem(
                new_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get component NodeIndex"),
            ),
            Conflict::NodeContent {
                onto: base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get component NodeIndex"),
                to_rebase: new_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get component NodeIndex"),
            },
        ];
        let expected_updates = vec![Update::ReplaceSubgraph {
            onto: base_graph
                .get_node_index_by_id(docker_image_schema_id)
                .expect("Unable to get NodeIndex"),
            to_rebase: new_graph
                .get_node_index_by_id(docker_image_schema_id)
                .expect("Unable to get NodeIndex"),
        }];

        assert_eq!(
            ConflictsAndUpdates {
                conflicts: expected_conflicts,
                updates: expected_updates,
            },
            ConflictsAndUpdates { conflicts, updates },
        );
    }

    #[test]
    fn add_ordered_node() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    func_id,
                    ContentAddress::Func(ContentHash::new(
                        FuncId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add func");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        PropId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph.cleanup();
        graph.dot();

        let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");
        graph.cleanup();
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
        let base_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let active_change_set = &base_change_set;
        let mut graph = WorkspaceSnapshotGraph::new(active_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let prop_id = active_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(
                active_change_set,
                NodeWeight::new_content(
                    active_change_set,
                    prop_id,
                    ContentAddress::Prop(ContentHash::new(prop_id.to_string().as_bytes())),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add prop");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(active_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add root -> prop edge");

        graph.cleanup();
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
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    func_id,
                    ContentAddress::Func(ContentHash::new(
                        FuncId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add func");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let prop_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        PropId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph.cleanup();
        graph.dot();

        let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");

        let ordered_prop_4_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_4_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add prop -> ordered_prop_4 edge");

        graph.cleanup();
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
            .update_order(change_set, prop_id, new_order)
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
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let schema_variant_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_2_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
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
            .mark_graph_seen(initial_change_set.vector_clock_id())
            .expect("Unable to mark initial graph as seen");

        let mut graph_with_deleted_edge = graph.clone();
        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;

        graph_with_deleted_edge.dot();

        graph_with_deleted_edge
            .remove_edge(
                new_change_set,
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

        graph_with_deleted_edge
            .mark_graph_seen(new_change_set.vector_clock_id())
            .expect("Unable to mark new graph as seen");

        let (conflicts, updates) = graph
            .detect_conflicts_and_updates(
                initial_change_set.vector_clock_id(),
                &graph_with_deleted_edge,
                new_change_set.vector_clock_id(),
            )
            .expect("Failed to detect conflicts and updates");

        assert!(conflicts.is_empty());
        dbg!(&updates);
        assert_eq!(1, updates.len());

        assert!(matches!(
            updates.first().expect("should be there"),
            Update::RemoveEdge { .. }
        ));
    }

    #[test]
    fn remove_unordered_node() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let schema_variant_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_2_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
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
                change_set,
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
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::new(
                        SchemaId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::new(
                        SchemaVariantId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");

        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let func_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    func_id,
                    ContentAddress::Func(ContentHash::new(
                        FuncId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add func");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                func_index,
            )
            .expect("Unable to add root -> func edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    root_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        PropId::generate().to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_index,
            )
            .expect("Unable to add schema variant -> prop edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(func_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add prop -> func edge");
        graph.cleanup();
        graph.dot();

        let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_1_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add prop -> ordered_prop_1 edge");

        let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_2_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add prop -> ordered_prop_2 edge");

        let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_3_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add prop -> ordered_prop_3 edge");

        let ordered_prop_4_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_4_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeWeight for prop"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create uses edge weight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add prop -> ordered_prop_4 edge");

        graph.cleanup();
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
                change_set,
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

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_no_conflicts_no_updates_in_base() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.cleanup();
        initial_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        let ordered_prop_5_id = new_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        new_graph
            .add_ordered_edge(
                new_change_set,
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");

        new_graph.cleanup();
        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &initial_graph,
                initial_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_no_conflicts_with_updates_in_base() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let new_graph = initial_graph.clone();

        let ordered_prop_5_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        let new_edge_weight = EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
            .expect("Unable to create EdgeWeight");
        let (_, maybe_ordinal_edge_information) = initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                new_edge_weight.clone(),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");
        let (
            ordinal_edge_index,
            source_node_index_for_ordinal_edge,
            destination_node_index_for_ordinal_edge,
        ) = maybe_ordinal_edge_information.expect("ordinal edge information not found");
        let ordinal_edge_weight = initial_graph
            .get_edge_weight_opt(ordinal_edge_index)
            .expect("should not error when getting edge")
            .expect("could not get edge weight for index")
            .to_owned();
        let source_node_id_for_ordinal_edge = initial_graph
            .get_node_weight(source_node_index_for_ordinal_edge)
            .expect("could not get node weight")
            .id();
        let destination_node_id_for_ordinal_edge = initial_graph
            .get_node_weight(destination_node_index_for_ordinal_edge)
            .expect("could not get node weight")
            .id();

        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &initial_graph,
                initial_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(
            vec![
                Update::NewEdge {
                    source: new_graph
                        .get_node_index_by_id(container_prop_id)
                        .expect("Unable to get NodeIndex"),
                    destination: initial_graph
                        .get_node_index_by_id(ordered_prop_5_id)
                        .expect("Unable to get NodeIndex"),
                    edge_weight: new_edge_weight,
                },
                Update::ReplaceSubgraph {
                    onto: initial_graph
                        .ordering_node_index_for_container(
                            initial_graph
                                .get_node_index_by_id(container_prop_id)
                                .expect("Unable to get container NodeIndex")
                        )
                        .expect("Unable to get new ordering NodeIndex")
                        .expect("Ordering NodeIndex not found"),
                    to_rebase: new_graph
                        .ordering_node_index_for_container(
                            new_graph
                                .get_node_index_by_id(container_prop_id)
                                .expect("Unable to get container NodeIndex")
                        )
                        .expect("Unable to get old ordering NodeIndex")
                        .expect("Ordering NodeIndex not found"),
                },
                Update::NewEdge {
                    source: new_graph
                        .get_node_index_by_id(source_node_id_for_ordinal_edge)
                        .expect("could not get node index by id"),
                    destination: initial_graph
                        .get_node_index_by_id(destination_node_id_for_ordinal_edge)
                        .expect("could not get node index by id"),
                    edge_weight: ordinal_edge_weight,
                }
            ],
            updates
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_with_conflicting_ordering_updates() {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        let new_order = vec![
            ordered_prop_2_id,
            ordered_prop_1_id,
            ordered_prop_4_id,
            ordered_prop_3_id,
        ];
        new_graph
            .update_order(new_change_set, container_prop_id, new_order)
            .expect("Unable to update order of container prop's children");

        let ordered_prop_5_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        let new_edge_weight = EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
            .expect("Unable to create EdgeWeight");
        let (_, maybe_ordinal_edge_information) = initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                new_edge_weight.clone(),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");
        let (
            ordinal_edge_index,
            source_node_index_for_ordinal_edge,
            destination_node_index_for_ordinal_edge,
        ) = maybe_ordinal_edge_information.expect("ordinal edge information not found");
        let ordinal_edge_weight = initial_graph
            .get_edge_weight_opt(ordinal_edge_index)
            .expect("should not error when getting edge")
            .expect("could not get edge weight for index")
            .to_owned();
        let source_node_id_for_ordinal_edge = initial_graph
            .get_node_weight(source_node_index_for_ordinal_edge)
            .expect("could not get node weight")
            .id();
        let destination_node_id_for_ordinal_edge = initial_graph
            .get_node_weight(destination_node_index_for_ordinal_edge)
            .expect("could not get node weight")
            .id();

        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &initial_graph,
                initial_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::ChildOrder {
                onto: initial_graph
                    .ordering_node_index_for_container(
                        initial_graph
                            .get_node_index_by_id(container_prop_id)
                            .expect("Unable to get container NodeIndex")
                    )
                    .expect("Unable to get ordering NodeIndex")
                    .expect("Ordering NodeIndex not found"),
                to_rebase: new_graph
                    .ordering_node_index_for_container(
                        new_graph
                            .get_node_index_by_id(container_prop_id)
                            .expect("Unable to get container NodeIndex")
                    )
                    .expect("Unable to get ordering NodeIndex")
                    .expect("Ordering NodeIndex not found"),
            }],
            conflicts
        );
        assert_eq!(
            vec![
                Update::NewEdge {
                    source: new_graph
                        .get_node_index_by_id(container_prop_id)
                        .expect("Unable to get new_graph container NodeIndex"),
                    destination: initial_graph
                        .get_node_index_by_id(ordered_prop_5_id)
                        .expect("Unable to get ordered prop 5 NodeIndex"),
                    edge_weight: new_edge_weight,
                },
                Update::NewEdge {
                    source: new_graph
                        .get_node_index_by_id(source_node_id_for_ordinal_edge)
                        .expect("could not get node index by id"),
                    destination: initial_graph
                        .get_node_index_by_id(destination_node_id_for_ordinal_edge)
                        .expect("could not get node index by id"),
                    edge_weight: ordinal_edge_weight,
                }
            ],
            updates
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_with_no_conflicts_add_in_onto_remove_in_to_rebase(
    ) {
        let initial_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let initial_change_set = &initial_change_set;
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_id,
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_graph.root_index,
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_change_set,
                NodeWeight::new_content(
                    initial_change_set,
                    container_prop_id,
                    ContentAddress::Prop(ContentHash::new(
                        container_prop_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add container prop");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_1_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_2_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_3_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_4_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.cleanup();
        initial_graph
            .mark_graph_seen(initial_change_set.vector_clock_id())
            .expect("Unable to update recently seen information");
        // initial_graph.dot();

        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;
        let mut new_graph = initial_graph.clone();

        new_graph
            .remove_edge(
                new_change_set,
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get container NodeIndex"),
                ordered_prop_2_index,
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to remove container prop -> prop 2 edge");

        let ordered_prop_5_id = initial_change_set
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_change_set,
                    ordered_prop_5_id,
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");

        let new_edge_weight = EdgeWeight::new(initial_change_set, EdgeWeightKind::new_use())
            .expect("Unable to create EdgeWeight");
        let (_, maybe_ordinal_edge_information) = initial_graph
            .add_ordered_edge(
                initial_change_set,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                new_edge_weight.clone(),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");
        let (
            ordinal_edge_index,
            source_node_index_for_ordinal_edge,
            destination_node_index_for_ordinal_edge,
        ) = maybe_ordinal_edge_information.expect("ordinal edge information not found");
        let ordinal_edge_weight = initial_graph
            .get_edge_weight_opt(ordinal_edge_index)
            .expect("should not error when getting edge")
            .expect("could not get edge weight for index")
            .to_owned();
        let source_node_id_for_ordinal_edge = initial_graph
            .get_node_weight(source_node_index_for_ordinal_edge)
            .expect("could not get node weight")
            .id();
        let destination_node_id_for_ordinal_edge = initial_graph
            .get_node_weight(destination_node_index_for_ordinal_edge)
            .expect("could not get node weight")
            .id();

        initial_graph.cleanup();
        initial_graph.dot();

        new_graph.cleanup();
        new_graph.dot();

        let (conflicts, updates) = new_graph
            .detect_conflicts_and_updates(
                new_change_set.vector_clock_id(),
                &initial_graph,
                initial_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(
            vec![
                Update::NewEdge {
                    source: new_graph
                        .get_node_index_by_id(container_prop_id)
                        .expect("Unable to get new_graph container NodeIndex"),
                    destination: initial_graph
                        .get_node_index_by_id(ordered_prop_5_id)
                        .expect("Unable to get ordered prop 5 NodeIndex"),
                    edge_weight: new_edge_weight,
                },
                Update::NewEdge {
                    source: new_graph
                        .get_node_index_by_id(source_node_id_for_ordinal_edge)
                        .expect("could not get node index by id"),
                    destination: initial_graph
                        .get_node_index_by_id(destination_node_id_for_ordinal_edge)
                        .expect("could not get node index by id"),
                    edge_weight: ordinal_edge_weight,
                }
            ],
            updates
        );
    }

    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let si_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_prop_content_hash = content_store
            .add(&serde_json::json!("SI Prop Content"))
            .expect("Unable to add to content store");
        let si_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    si_prop_id,
                    PropKind::Object,
                    "si",
                    si_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                si_prop_node_index,
            )
            .expect("Unable to add root prop -> si prop edge");

        let name_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_prop_content_hash = content_store
            .add(&serde_json::json!("Name Prop Content"))
            .expect("Unable to add to content store");
        let name_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    name_prop_id,
                    PropKind::Object,
                    "name",
                    name_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                name_prop_node_index,
            )
            .expect("Unable to add si prop -> name prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let si_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let si_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    si_av_id,
                    ContentAddress::AttributeValue(si_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                si_av_node_index,
            )
            .expect("Unable to add root av -> si av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add si av -> si prop edge");

        let name_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_av_content_hash = content_store
            .add(&serde_json::json!("component name"))
            .expect("Unable to add to content store");
        let name_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    name_av_id,
                    ContentAddress::AttributeValue(name_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeWeight"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                name_av_node_index,
            )
            .expect("Unable to add si av -> name av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(name_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(name_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create name av -> name prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{"si": {"name": "component name"}}],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }

    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view_unordered_object() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let si_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_prop_content_hash = content_store
            .add(&serde_json::json!("SI Prop Content"))
            .expect("Unable to add to content store");
        let si_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    si_prop_id,
                    PropKind::Object,
                    "si",
                    si_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                si_prop_node_index,
            )
            .expect("Unable to add root prop -> si prop edge");

        let name_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_prop_content_hash = content_store
            .add(&serde_json::json!("Name Prop Content"))
            .expect("Unable to add to content store");
        let name_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    name_prop_id,
                    PropKind::Object,
                    "name",
                    name_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                name_prop_node_index,
            )
            .expect("Unable to add si prop -> name prop edge");

        let description_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let description_prop_content_hash = content_store
            .add(&serde_json::json!("Description Prop Content"))
            .expect("Unable to add to content store");
        let description_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    description_prop_id,
                    PropKind::String,
                    "description",
                    description_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add description prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                description_prop_node_index,
            )
            .expect("Unable to add si prop -> description prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let si_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let si_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    si_av_id,
                    ContentAddress::AttributeValue(si_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                si_av_node_index,
            )
            .expect("Unable to add root av -> si av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add si av -> si prop edge");

        let name_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_av_content_hash = content_store
            .add(&serde_json::json!("component name"))
            .expect("Unable to add to content store");
        let name_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    name_av_id,
                    ContentAddress::AttributeValue(name_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                name_av_node_index,
            )
            .expect("Unable to add si av -> name av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(name_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(name_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create name av -> name prop edge");

        let description_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let description_av_content_hash = content_store
            .add(&serde_json::json!("Component description"))
            .expect("Unable to add to content store");
        let description_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    description_av_id,
                    ContentAddress::AttributeValue(description_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add description av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                description_av_node_index,
            )
            .expect("Unable to add si av -> description av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(description_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(description_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add description av -> description prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "si": {
                    "description": "Component description",
                    "name": "component name",
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }

    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view_ordered_array() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let domain_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_prop_content_hash = content_store
            .add(&serde_json::json!("domain Prop Content"))
            .expect("Unable to add to content store");
        let domain_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    domain_prop_id,
                    PropKind::Object,
                    "domain",
                    domain_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                domain_prop_node_index,
            )
            .expect("Unable to add root prop -> domain prop edge");

        let ports_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ports_prop_content_hash = content_store
            .add(&serde_json::json!("ports Prop Content"))
            .expect("Unable to add to content store");
        let ports_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    ports_prop_id,
                    PropKind::Array,
                    "ports",
                    ports_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ports prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ports_prop_node_index,
            )
            .expect("Unable to add domain prop -> ports prop edge");

        let port_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port_prop_content_hash = content_store
            .add(&serde_json::json!("port Prop Content"))
            .expect("Unable to add to content store");
        let port_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    port_prop_id,
                    PropKind::String,
                    "port",
                    port_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(ports_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                port_prop_node_index,
            )
            .expect("Unable to add ports prop -> port prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let domain_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let domain_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    domain_av_id,
                    ContentAddress::AttributeValue(domain_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                domain_av_node_index,
            )
            .expect("Unable to add root av -> domain av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add domain av -> domain prop edge");

        let ports_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ports_av_content_hash = content_store
            .add(&serde_json::json!([]))
            .expect("Unable to add to content store");
        let ports_av_node_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    ports_av_id,
                    ContentAddress::AttributeValue(ports_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ports av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                ports_av_node_index,
            )
            .expect("Unable to add domain av -> ports av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(ports_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create ports av -> ports prop edge");

        let port1_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port1_av_content_hash = content_store
            .add(&serde_json::json!("Port 1"))
            .expect("Unable to add to content store");
        let port1_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port1_av_id,
                    ContentAddress::AttributeValue(port1_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 1 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port1_av_node_index,
            )
            .expect("Unable to add ports av -> port 1 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port1_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 1 av -> port prop edge");

        let port2_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port2_av_content_hash = content_store
            .add(&serde_json::json!("Port 2"))
            .expect("Unable to add to content store");
        let port2_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port2_av_id,
                    ContentAddress::AttributeValue(port2_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 2 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port2_av_node_index,
            )
            .expect("Unable to add ports av -> port 2 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port2_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 2 av -> port prop edge");

        let port3_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port3_av_content_hash = content_store
            .add(&serde_json::json!("Port 3"))
            .expect("Unable to add to content store");
        let port3_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port3_av_id,
                    ContentAddress::AttributeValue(port3_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 3 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port3_av_node_index,
            )
            .expect("Unable to add ports av -> port 3 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port3_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 3 av -> port prop edge");

        let port4_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port4_av_content_hash = content_store
            .add(&serde_json::json!("Port 4"))
            .expect("Unable to add to content store");
        let port4_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port4_av_id,
                    ContentAddress::AttributeValue(port4_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 4 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port4_av_node_index,
            )
            .expect("Unable to add ports av -> port 4 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port4_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 4 av -> port prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 1",
                        "Port 2",
                        "Port 3",
                        "Port 4",
                    ],
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let new_order = vec![port3_av_id, port1_av_id, port4_av_id, port2_av_id];
        graph
            .update_order(change_set, ports_av_id, new_order)
            .expect("Unable to update order of ports attribute value's children");
        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 3",
                        "Port 1",
                        "Port 4",
                        "Port 2",
                    ]
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let port5_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port5_av_content_hash = content_store
            .add(&serde_json::json!("Port 5"))
            .expect("Unable to add to content store");
        let port5_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port5_av_id,
                    ContentAddress::AttributeValue(port5_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 5 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port5_av_node_index,
            )
            .expect("Unable to add ports av -> port 5 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port5_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 5 av -> port prop edge");

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 3",
                        "Port 1",
                        "Port 4",
                        "Port 2",
                        "Port 5",
                    ]
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }

    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view_ordered_map() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let domain_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_prop_content_hash = content_store
            .add(&serde_json::json!("domain Prop Content"))
            .expect("Unable to add to content store");
        let domain_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    domain_prop_id,
                    PropKind::Object,
                    "domain",
                    domain_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                domain_prop_node_index,
            )
            .expect("Unable to add root prop -> domain prop edge");

        let environment_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let environment_prop_content_hash = content_store
            .add(&serde_json::json!("environment Prop Content"))
            .expect("Unable to add to content store");
        let environment_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    environment_prop_id,
                    PropKind::Array,
                    "environment",
                    environment_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add environment prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                environment_prop_node_index,
            )
            .expect("Unable to add domain prop -> environment prop edge");

        let env_var_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var_prop_content_hash = content_store
            .add(&serde_json::json!("port Prop Content"))
            .expect("Unable to add to content store");
        let env_var_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    env_var_prop_id,
                    PropKind::String,
                    "port",
                    env_var_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(environment_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                env_var_prop_node_index,
            )
            .expect("Unable to add environment prop -> env var prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_attribute_value(change_set, root_av_id, None, None, None)
                    .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let domain_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let domain_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    domain_av_id,
                    ContentAddress::AttributeValue(domain_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                domain_av_node_index,
            )
            .expect("Unable to add root av -> domain av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add domain av -> domain prop edge");

        let envrionment_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ports_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let environment_av_node_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    envrionment_av_id,
                    ContentAddress::AttributeValue(ports_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add environment av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                environment_av_node_index,
            )
            .expect("Unable to add domain av -> environment av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(environment_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create environment av -> environment prop edge");

        let env_var1_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var1_av_content_hash = content_store
            .add(&serde_json::json!("1111"))
            .expect("Unable to add to content store");
        let port1_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var1_av_id,
                    ContentAddress::AttributeValue(env_var1_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env_var 1 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_1".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                port1_av_node_index,
            )
            .expect("Unable to add environment av -> env var 1 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var1_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 1 av -> env var prop edge");

        let env_var2_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var2_av_content_hash = content_store
            .add(&serde_json::json!("2222"))
            .expect("Unable to add to content store");
        let env_var2_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var2_av_id,
                    ContentAddress::AttributeValue(env_var2_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 2 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_2".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var2_av_node_index,
            )
            .expect("Unable to add environment av -> env var 2 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var2_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 2 av -> env var prop edge");

        let env_var3_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var3_av_content_hash = content_store
            .add(&serde_json::json!("3333"))
            .expect("Unable to add to content store");
        let port3_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var3_av_id,
                    ContentAddress::AttributeValue(env_var3_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 3 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_3".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                port3_av_node_index,
            )
            .expect("Unable to add environment av -> env var 3 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var3_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 3 av -> env var prop edge");

        let env_var4_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var4_av_content_hash = content_store
            .add(&serde_json::json!("4444"))
            .expect("Unable to add to content store");
        let env_var4_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var4_av_id,
                    ContentAddress::AttributeValue(env_var4_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 4 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_4".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var4_av_node_index,
            )
            .expect("Unable to add environment av -> env var 4 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var4_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 4 av -> env var prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_1": "1111",
                        "PORT_2": "2222",
                        "PORT_3": "3333",
                        "PORT_4": "4444",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let new_order = vec![
            env_var3_av_id,
            env_var1_av_id,
            env_var4_av_id,
            env_var2_av_id,
        ];
        graph
            .update_order(change_set, envrionment_av_id, new_order)
            .expect("Unable to update order of environment attribute value's children");
        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_3": "3333",
                        "PORT_1": "1111",
                        "PORT_4": "4444",
                        "PORT_2": "2222",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let env_var5_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var5_av_content_hash = content_store
            .add(&serde_json::json!("5555"))
            .expect("Unable to add to content store");
        let env_var5_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var5_av_id,
                    ContentAddress::AttributeValue(env_var5_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 5 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_5".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var5_av_node_index,
            )
            .expect("Unable to add environment av -> env var 5 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var5_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 5 av -> env var prop edge");

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_3": "3333",
                        "PORT_1": "1111",
                        "PORT_4": "4444",
                        "PORT_2": "2222",
                        "PORT_5": "5555",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }
}
