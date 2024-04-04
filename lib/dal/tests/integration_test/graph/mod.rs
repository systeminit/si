mod rebase;

use dal::workspace_snapshot::content_address::ContentAddress;
use dal::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use dal::ComponentId;
use dal::DalContext;
use dal::FuncId;
use dal::PropId;
use dal::SchemaId;
use dal::SchemaVariantId;
use dal::WorkspaceSnapshot;
use petgraph::graph::NodeIndex;
use petgraph::Outgoing;
use si_events::ContentHash;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use ulid::Ulid;

use dal::change_set::ChangeSet;
use dal::workspace_snapshot::conflict::Conflict;
use dal::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use dal::workspace_snapshot::node_weight::NodeWeight;
use dal::workspace_snapshot::update::Update;
use dal::PropKind;
use dal_test::test;
use si_events::MerkleTreeHash;

#[derive(Debug, PartialEq)]
struct ConflictsAndUpdates {
    conflicts: Vec<Conflict>,
    updates: Vec<Update>,
}

// #[test]
// async fn new(ctx: DalContext) {
//     let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
//     let change_set = &change_set;
//     let graph =
//         WorkspaceSnapshotGraph::new(change_set).expect("Unable to create WorkspaceSnapshotGraph");
//     assert!(graph.is_acyclic_directed());
// }

// Previously, WorkspaceSnapshotGraph::new would not populate its node_index_by_id, so this test
// would fail, in addition to any functionality that depended on getting the root node index
// on a fresh graph (like add_ordered_node)
#[test]
async fn get_root_index_by_root_id_on_fresh_graph(ctx: &DalContext) {
    let base_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let active_change_set = &base_change_set;
    let graph = WorkspaceSnapshot::empty(ctx, active_change_set)
        .await
        .expect("Unable to create WorkspaceSnapshot");

    let root_id = graph.root_id().await.expect("Unable to get rootId");

    let root_node_idx = graph
        .get_node_index_by_id(root_id)
        .await
        .expect("get root node index from ULID");

    assert_eq!(
        graph.root().await.expect("Unable to get root idx"),
        root_node_idx
    );
}

#[test]
async fn multiply_parented_nodes(ctx: DalContext) {
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
    let graph = WorkspaceSnapshot::empty(&ctx, change_set)
        .await
        .expect("should create snapshot");
    let root_id = graph.root_id().await.expect("should get root id");

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
            .await
            .expect("Unable to add prop");

        node_id_map.insert(node, node_id);
    }

    for (source, target) in edges {
        let source = match source {
            None => root_id,
            Some(node) => node_id_map.get(node).copied().expect("should be there"),
        };

        let target = node_id_map
            .get(target)
            .copied()
            .expect("target node should have an id");

        graph
            .add_edge(
                source,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use()).expect("create edge weight"),
                target,
            )
            .await
            .expect("add edge");
    }

    graph.cleanup().await.expect("should cleanup");

    for (source, target) in edges {
        let source = match source {
            None => root_id,
            Some(node) => node_id_map.get(node).copied().expect("should be there"),
        };

        let target_idx = graph
            .get_node_index_by_id(
                node_id_map
                    .get(target)
                    .copied()
                    .expect("target node should have an id"),
            )
            .await
            .expect("get node index by id");

        assert!(
            graph
                .edges_directed(source, Outgoing)
                .await
                .expect("should be able to get edges directed")
                .iter()
                .any(|(_, _, target)| target == &target_idx),
            "An edge from {} to {} should exist",
            source,
            target
        );
    }

    for (_, id) in node_id_map.iter() {
        let idx_for_node = graph
            .get_node_index_by_id(*id)
            .await
            .expect("able to get idx by id");
        graph
            .get_node_weight(idx_for_node)
            .await
            .expect("node with weight in graph");
    }
}

#[test]
async fn add_nodes_and_edges(ctx: DalContext) {
    let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let change_set = &change_set;
    let graph = WorkspaceSnapshot::empty(&ctx, change_set)
        .await
        .expect("should create snapshot");
    let root_id = graph.root_id().await.expect("should get root id");

    let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
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
        .await
        .expect("Unable to add schema");
    let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
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
        .await
        .expect("Unable to add schema variant");
    let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
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
        .await
        .expect("Unable to add component");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");
    graph
        .add_edge(
            component_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                func_id,
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add func");
    let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                prop_id,
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add prop");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add root -> func edge");
    graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            prop_id,
        )
        .await
        .expect("Unable to add schema variant -> prop edge");
    graph
        .add_edge(
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add prop -> func edge");

    assert!(graph.is_acyclic_directed().await);
}

#[test]
async fn cyclic_failure(ctx: DalContext) {
    let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let change_set = &change_set;
    let graph = WorkspaceSnapshot::empty(&ctx, change_set)
        .await
        .expect("should create snapshot");
    let root_id = graph.root_id().await.expect("should get root id");

    let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
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
        .await
        .expect("Unable to add node");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> component edge");

    let pre_cycle_root_index = root_id;

    println!("before cycle check");
    // This should cause a cycle.
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            root_id,
        )
        .await
        .expect_err("Created a cycle");
    println!("after cycle check");

    let current_root_id = graph.root_id().await.expect("should get root id");

    assert_eq!(pre_cycle_root_index, current_root_id,);
}

#[test]
async fn update_content(ctx: DalContext) {
    let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let change_set = &change_set;
    let graph = WorkspaceSnapshot::empty(&ctx, change_set)
        .await
        .expect("should create snapshot");
    let root_id = graph.root_id().await.expect("should get root id");

    let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Constellation")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add schema");
    let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::new("Freestar Collective".as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add schema variant");
    let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                component_id,
                ContentAddress::Component(ContentHash::from("Crimson Fleet")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add component");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");
    graph
        .add_edge(
            component_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("should be able to calculate the merkle tree hash");

    // Ensure that the root node merkle tree hash looks as we expect before the update.
    let pre_update_root_node_merkle_tree_hash = MerkleTreeHash::from_str(
        "6b3b0374a25049046f34d6c7e98f890387a963249aaace3d66bb47ce70399033",
    )
    .expect("could not make merkle tree hash from hex bytes");
    assert_eq!(
        pre_update_root_node_merkle_tree_hash, // expected
        graph
            .get_graph_local_node_weight(graph.root_id().await.expect("get root"))
            .await
            .expect("could not get node weight")
            .merkle_tree_hash(), // actual
    );

    let updated_content_hash = ContentHash::from("new_content");
    graph
        .update_content(change_set, component_id, updated_content_hash)
        .await
        .expect("Unable to update Component content hash");

    graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("should be able to calculate the merkle tree hash");

    let post_update_root_node_merkle_tree_hash = MerkleTreeHash::from_str(
        "46babffabf1567fd20594c7038cfea58991b394b8eb6cc1f81167d2314617e35",
    )
    .expect("merkle hash from str");
    assert_eq!(
        post_update_root_node_merkle_tree_hash, // expected
        graph
            .get_graph_local_node_weight(graph.root_id().await.expect("get root"))
            .await
            .expect("could not get node weight")
            .merkle_tree_hash(), // actual
    );
    assert_eq!(
        updated_content_hash, // expected
        graph
            .get_node_weight(
                graph
                    .get_node_index_by_id(component_id)
                    .await
                    .expect("could not get node index by id")
            )
            .await
            .expect("could not get node weight")
            .content_hash(), // actual
    );

    graph.cleanup().await.expect("should cleanup");

    // Ensure that there are not more nodes than the ones that should be in use.
    assert_eq!(4, graph.node_count().await);

    // The hashes must not change upon cleanup.
    assert_eq!(
        post_update_root_node_merkle_tree_hash, // expected
        graph
            .get_graph_local_node_weight(graph.root_id().await.expect("get root"))
            .await
            .expect("could not get node weight")
            .merkle_tree_hash()
    );
    assert_eq!(
        updated_content_hash, // expected
        graph
            .get_node_weight(
                graph
                    .get_node_index_by_id(component_id)
                    .await
                    .expect("could not get node index by id")
            )
            .await
            .expect("could not get node weight")
            .content_hash(), // actual
    );
}

#[test]
async fn detect_conflicts_and_updates_simple_no_conflicts_no_updates_in_base(ctx: DalContext) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let empty_change_set = &empty_change_set;
    let empty_graph = WorkspaceSnapshot::initial(&ctx, empty_change_set)
        .await
        .expect("should create snapshot");
    let empty_root_id = empty_graph.root_id().await.expect("should get root id");

    let schema_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    empty_graph
        .add_edge(
            empty_root_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    empty_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    // empty_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = empty_graph.real_clone().await;
    let new_root_id = empty_graph.root_id().await.expect("should get root id");

    let component_id = new_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    new_graph
        .add_node(
            NodeWeight::new_content(
                new_change_set,
                component_id,
                ContentAddress::Schema(ContentHash::from("Component A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component A");
    new_graph
        .add_edge(
            new_root_id,
            EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    new_graph
        .add_edge(
            component_id,
            EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    // new_graph.dot();

    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mthash");
    empty_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &empty_graph,
            empty_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert_eq!(Vec::<Conflict>::new(), conflicts);
    assert_eq!(Vec::<Update>::new(), updates);
}

#[test]
async fn detect_conflicts_and_updates_simple_no_conflicts_with_purely_new_content_in_base(
    ctx: DalContext,
) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let base_change_set = &empty_change_set;
    let base_graph = WorkspaceSnapshot::empty(&ctx, base_change_set)
        .await
        .expect("should create snapshot");
    let base_root_id = base_graph.root_id().await.expect("should get root id");

    let schema_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    base_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    println!("Initial base graph (Root {:?}):", base_root_id);
    // base_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = base_graph.real_clone().await;
    let new_root_id = new_graph.root_id().await.expect("should get root id");

    let new_onto_component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                new_onto_component_id,
                ContentAddress::Component(ContentHash::from("Component B")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component B");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            new_onto_component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    base_graph
        .add_edge(
            new_onto_component_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    println!("Updated base graph (Root: {:?}):", new_root_id);
    // base_graph.dot();

    base_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &base_graph,
            base_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert_eq!(Vec::<Conflict>::new(), conflicts);

    let new_onto_component_index = base_graph
        .get_node_index_by_id(new_onto_component_id)
        .await
        .expect("Unable to get NodeIndex");
    match updates.as_slice() {
        [Update::NewEdge {
            source,
            destination,
            edge_weight,
        }] => {
            assert_eq!(
                new_graph.root().await.expect("should get root index"),
                *source
            );
            assert_eq!(new_onto_component_index, *destination);
            assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
        }
        other => panic!("Unexpected updates: {:?}", other),
    }
}

#[test]
async fn detect_conflicts_and_updates_with_purely_new_content_in_new_graph(ctx: DalContext) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let base_change_set = &empty_change_set;
    let base_graph = WorkspaceSnapshot::empty(&ctx, base_change_set)
        .await
        .expect("should create snapshot");
    let base_root_id = base_graph.root_id().await.expect("should get root id");

    let component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                component_id,
                ContentAddress::Component(ContentHash::from("Component A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            component_id,
        )
        .await
        .expect("Unable to add root -> component edge");

    base_graph.cleanup().await.expect("should cleanup");
    println!("Initial base graph (Root {:?}):", base_root_id);
    // base_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = base_graph.real_clone().await;
    let new_root_id = new_graph.root_id().await.expect("should get root id");

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
        .await
        .expect("Unable to add Component B");
    new_graph
        .add_edge(
            new_root_id,
            EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            new_component_id,
        )
        .await
        .expect("Unable to add root -> component edge");

    new_graph.cleanup().await.expect("should clean up");
    println!("Updated new graph (Root: {:?}):", new_root_id);
    // new_graph.dot();
    base_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &base_graph,
            base_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert!(updates.is_empty());
    assert!(conflicts.is_empty());

    let (conflicts, updates) = base_graph
        .detect_conflicts_and_updates(
            base_change_set.vector_clock_id(),
            &new_graph,
            new_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert!(conflicts.is_empty());

    match updates.as_slice() {
        [Update::NewEdge {
            source,
            destination,
            edge_weight,
        }] => {
            assert_eq!(
                base_graph.root().await.expect("should get root index"),
                *source
            );
            assert_eq!(new_component_index, *destination);
            assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
        }
        other => panic!("Unexpected updates: {:?}", other),
    }
}

#[test]
async fn detect_conflicts_and_updates_simple_no_conflicts_with_updates_on_both_sides(
    ctx: DalContext,
) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let base_change_set = &empty_change_set;
    let base_graph = WorkspaceSnapshot::empty(&ctx, base_change_set)
        .await
        .expect("should create snapshot");
    let base_root_id = base_graph.root_id().await.expect("should get root id");

    let schema_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    base_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    println!("Initial base graph (Root {:?}):", base_root_id);
    // base_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = base_graph.real_clone().await;
    let new_root_id = new_graph.root_id().await.expect("should get root id");

    let component_id = new_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    new_graph
        .add_node(
            NodeWeight::new_content(
                new_change_set,
                component_id,
                ContentAddress::Component(ContentHash::from("Component A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component A");
    new_graph
        .add_edge(
            new_root_id,
            EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    new_graph
        .add_edge(
            component_id,
            EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    println!("new graph (Root {:?}):", new_root_id);
    // new_graph.dot();

    let new_onto_component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                new_onto_component_id,
                ContentAddress::Component(ContentHash::from("Component B")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component B");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            new_onto_component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    base_graph
        .add_edge(
            new_onto_component_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    println!("Updated base graph (Root: {:?}):", base_root_id);
    base_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &base_graph,
            base_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert_eq!(Vec::<Conflict>::new(), conflicts);

    let new_onto_component_index = base_graph
        .get_node_index_by_id(new_onto_component_id)
        .await
        .expect("Unable to get NodeIndex");
    match updates.as_slice() {
        [Update::NewEdge {
            source,
            destination,
            edge_weight,
        }] => {
            assert_eq!(
                new_graph.root().await.expect("should get root index"),
                *source
            );
            assert_eq!(new_onto_component_index, *destination);
            assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
        }
        other => panic!("Unexpected updates: {:?}", other),
    }
}

#[test]
async fn detect_conflicts_and_updates_simple_with_content_conflict(ctx: DalContext) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let base_change_set = &empty_change_set;
    let base_graph = WorkspaceSnapshot::empty(&ctx, base_change_set)
        .await
        .expect("should create snapshot");
    let base_root_id = base_graph.root_id().await.expect("should get root id");

    let schema_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    base_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                component_id,
                ContentAddress::Component(ContentHash::from("Component A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component A");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    base_graph
        .add_edge(
            component_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    base_graph.cleanup().await.expect("should clean up");
    println!("Initial base graph (Root {:?}):", base_root_id);
    // base_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = base_graph.real_clone().await;
    let _new_root_id = new_graph.root_id().await.expect("should get root id");

    new_graph
        .update_content(
            new_change_set,
            component_id,
            ContentHash::from("Updated Component A"),
        )
        .await
        .expect("Unable to update Component A");

    new_graph.cleanup().await.expect("should clean up");
    // new_graph.dot();

    base_graph
        .update_content(
            base_change_set,
            component_id,
            ContentHash::from("Base Updated Component A"),
        )
        .await
        .expect("Unable to update Component A");
    // new_graph.dot();

    base_graph.cleanup().await.expect("should clean up");

    // base_graph.tiny_dot_to_file(Some("base_graph")).await;
    // new_graph.tiny_dot_to_file(Some("new_graph")).await;
    //
    base_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &base_graph,
            base_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert_eq!(
        vec![Conflict::NodeContent {
            onto: base_graph
                .get_node_index_by_id(component_id)
                .await
                .expect("Unable to get component NodeIndex"),
            to_rebase: new_graph
                .get_node_index_by_id(component_id)
                .await
                .expect("Unable to get component NodeIndex"),
        }],
        conflicts
    );
    assert_eq!(Vec::<Update>::new(), updates);
}

#[test]
async fn detect_conflicts_and_updates_simple_with_modify_removed_item_conflict(ctx: &DalContext) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let base_change_set = &empty_change_set;
    let base_graph = WorkspaceSnapshot::empty(ctx, base_change_set)
        .await
        .expect("should create snapshot");
    let base_root_id = base_graph.root_id().await.expect("should get root id");

    let schema_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");

    let schema_variant_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    base_graph
        .add_edge(
            dbg!(base_root_id),
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            dbg!(schema_id),
        )
        .await
        .expect("Unable to add root -> schema edge");
    base_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let _component_index = base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                component_id,
                ContentAddress::Component(ContentHash::from("Component A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component A");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    base_graph
        .add_edge(
            component_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    base_graph.cleanup().await.expect("should clean up");

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let base_root_id = base_graph.root_id().await.expect("get root id");
    let new_graph = base_graph.real_clone().await;

    base_graph
        .remove_edge_for_ulids(
            base_change_set,
            base_root_id,
            component_id,
            EdgeWeightKindDiscriminants::Use,
        )
        .await
        .expect("Unable to remove Component A");

    base_graph.cleanup().await.expect("should clean up");

    new_graph
        .update_content(
            new_change_set,
            component_id,
            ContentHash::from("Updated Component A"),
        )
        .await
        .expect("Unable to update Component A");

    new_graph.cleanup().await.expect("should clean up");

    base_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &base_graph,
            base_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert_eq!(
        vec![Conflict::ModifyRemovedItem(
            new_graph
                .get_node_index_by_id(component_id)
                .await
                .expect("Unable to get NodeIndex")
        )],
        conflicts
    );
    assert_eq!(Vec::<Update>::new(), updates);
}

#[test]
async fn detect_conflicts_and_updates_complex(ctx: &DalContext) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let base_change_set = &empty_change_set;
    let base_graph = WorkspaceSnapshot::empty(ctx, base_change_set)
        .await
        .expect("Unable to create WorkspaceSnapshotGraph");
    let base_root_id = base_graph.root_id().await.expect("unable to get root id");

    // Docker Image Schema
    let docker_image_schema_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let _docker_image_schema_index = base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                docker_image_schema_id,
                ContentAddress::Schema(ContentHash::from("first")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");

    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            docker_image_schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");

    // Docker Image Schema Variant
    let docker_image_schema_variant_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                docker_image_schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("first")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");
    base_graph
        .add_edge(
            docker_image_schema_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            docker_image_schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    // Nginx Docker Image Component
    let nginx_docker_image_component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let _nginx_docker_image_component_index = base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                nginx_docker_image_component_id,
                ContentAddress::Component(ContentHash::from("first")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component A");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            nginx_docker_image_component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    base_graph
        .add_edge(
            nginx_docker_image_component_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            docker_image_schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    // Alpine Component
    let alpine_component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                alpine_component_id,
                ContentAddress::Component(ContentHash::from("first")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Component A");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            alpine_component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    base_graph
        .add_edge(
            alpine_component_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            docker_image_schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    // Butane Schema
    let butane_schema_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let _butane_schema_index = base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                butane_schema_id,
                ContentAddress::Schema(ContentHash::from("first")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            butane_schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");

    // Butane Schema Variant
    let butane_schema_variant_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                butane_schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("first")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");
    base_graph
        .add_edge(
            butane_schema_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            butane_schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    // Nginx Butane Component
    let nginx_butane_component_id = base_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let _nginx_butane_node_index = base_graph
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                nginx_butane_component_id,
                ContentAddress::Component(ContentHash::from("first")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");
    base_graph
        .add_edge(
            base_root_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            nginx_butane_component_id,
        )
        .await
        .expect("Unable to add root -> component edge");
    base_graph
        .add_edge(
            nginx_butane_component_id,
            EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            butane_schema_variant_id,
        )
        .await
        .expect("Unable to add component -> schema variant edge");

    base_graph.cleanup().await.expect("should clean up");
    base_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("should calculate entire merkle tree hash");

    // Create a new change set to cause some problems!
    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = base_graph.real_clone().await;

    // Create a modify removed item conflict.
    base_graph
        .remove_edge_for_ulids(
            base_change_set,
            base_root_id,
            nginx_butane_component_id,
            EdgeWeightKindDiscriminants::Use,
        )
        .await
        .expect("Unable to update the component");
    new_graph
        .update_content(
            new_change_set,
            nginx_butane_component_id,
            ContentHash::from("second"),
        )
        .await
        .expect("Unable to update the component");

    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("should calculate mth");

    // Create a node content conflict.
    base_graph
        .update_content(
            base_change_set,
            docker_image_schema_variant_id,
            ContentHash::from("oopsie"),
        )
        .await
        .expect("Unable to update the component");
    new_graph
        .update_content(
            new_change_set,
            docker_image_schema_variant_id,
            ContentHash::from("poopsie"),
        )
        .await
        .expect("Unable to update the component");

    // Create a pure update.
    base_graph
        .update_content(
            base_change_set,
            docker_image_schema_id,
            ContentHash::from("bg3"),
        )
        .await
        .expect("Unable to update the schema");

    base_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("should be able to calculate merkle tree hash");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &base_graph,
            base_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    let expected_conflicts = vec![
        Conflict::ModifyRemovedItem(
            new_graph
                .get_node_index_by_id(nginx_butane_component_id)
                .await
                .expect("Unable to get component NodeIndex"),
        ),
        Conflict::NodeContent {
            onto: base_graph
                .get_node_index_by_id(docker_image_schema_variant_id)
                .await
                .expect("Unable to get component NodeIndex"),
            to_rebase: new_graph
                .get_node_index_by_id(docker_image_schema_variant_id)
                .await
                .expect("Unable to get component NodeIndex"),
        },
    ];
    let expected_updates = vec![Update::ReplaceSubgraph {
        onto: base_graph
            .get_node_index_by_id(docker_image_schema_id)
            .await
            .expect("Unable to get NodeIndex"),
        to_rebase: new_graph
            .get_node_index_by_id(docker_image_schema_id)
            .await
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
async fn add_ordered_node(ctx: &DalContext) {
    let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let change_set = &change_set;
    let graph = WorkspaceSnapshot::empty(ctx, change_set)
        .await
        .expect("Unable to create WorkspaceSnapshotGraph");

    let root_id = graph
        .root_id()
        .await
        .expect("couldn't get root id for graph");

    let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _schema_index = graph
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
        .await
        .expect("Unable to add schema");
    let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _schema_variant_index = graph
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
        .await
        .expect("Unable to add schema variant");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _func_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                func_id,
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add func");
    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add root -> func edge");

    let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _prop_index = graph
        .add_ordered_node(
            change_set,
            NodeWeight::new_content(
                change_set,
                prop_id,
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add prop");
    graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            prop_id,
        )
        .await
        .expect("Unable to add schema variant -> prop edge");
    graph
        .add_edge(
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add prop -> func edge");
    graph.cleanup().await.expect("should clean up");

    let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _ordered_prop_1_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_1_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_1_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_1 edge");

    let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _ordered_prop_2_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_2_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_2_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_2 edge");

    let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _ordered_prop_3_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_3_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_3_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_3 edge");
    graph.cleanup().await.expect("should clean up");

    assert_eq!(
        vec![ordered_prop_1_id, ordered_prop_2_id, ordered_prop_3_id,],
        graph
            .ordered_children_for_node(prop_id)
            .await
            .expect("Unable to find ordered children for node")
            .expect("Node is not an ordered node")
    );
}

#[test]
async fn reorder_ordered_node(ctx: &DalContext) {
    let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let change_set = &change_set;
    let graph = WorkspaceSnapshot::empty(ctx, change_set)
        .await
        .expect("Unable to create WorkspaceSnapshotGraph");

    let root_id = graph.root_id().await.expect("get root id");
    let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _schema_index = graph
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
        .await
        .expect("Unable to add schema");
    let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _schema_variant_index = graph
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
        .await
        .expect("Unable to add schema variant");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _func_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                func_id,
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add func");
    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add root -> func edge");

    let prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _prop_index = graph
        .add_ordered_node(
            change_set,
            NodeWeight::new_content(
                change_set,
                prop_id,
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add prop");
    graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            prop_id,
        )
        .await
        .expect("Unable to add schema variant -> prop edge");
    graph
        .add_edge(
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add prop -> func edge");
    graph.cleanup().await.expect("should clean up");

    let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _ordered_prop_1_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_1_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_1_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_1 edge");

    let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _ordered_prop_2_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_2_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_2_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_2 edge");

    let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _ordered_prop_3_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_3_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_3_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_3 edge");

    let ordered_prop_4_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _ordered_prop_4_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_4_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_4_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_4 edge");

    graph.cleanup().await.expect("should clean up");

    assert_eq!(
        vec![
            ordered_prop_1_id,
            ordered_prop_2_id,
            ordered_prop_3_id,
            ordered_prop_4_id,
        ],
        graph
            .ordered_children_for_node(prop_id)
            .await
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
        .await
        .expect("Unable to update order of prop's children");

    assert_eq!(
        vec![
            ordered_prop_2_id,
            ordered_prop_1_id,
            ordered_prop_4_id,
            ordered_prop_3_id,
        ],
        graph
            .ordered_children_for_node(prop_id)
            .await
            .expect("Unable to find ordered children for node")
            .expect("Node is not an ordered node")
    );
}
//
#[test]
async fn remove_unordered_node_and_detect_edge_removal(ctx: &DalContext) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let empty_change_set = &empty_change_set;
    let graph = WorkspaceSnapshot::empty(ctx, empty_change_set)
        .await
        .expect("Unable to create WorkspaceSnapshotGraph");

    let root_id = graph.root_id().await.expect("unable to get root id");

    let schema_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let _schema_index = graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::new(
                    SchemaId::generate().to_string().as_bytes(),
                )),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add schema");
    let schema_variant_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let schema_variant_index = graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add schema variant");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let schema_variant_2_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    let schema_variant_2_index = graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_variant_2_id,
                ContentAddress::SchemaVariant(ContentHash::new(
                    SchemaVariantId::generate().to_string().as_bytes(),
                )),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add schema variant");

    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_2_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let expected_edges = HashSet::from([schema_variant_2_index, schema_variant_index]);

    let existing_edges: HashSet<NodeIndex> = graph
        .edges_directed(schema_id, Outgoing)
        .await
        .expect("able to get edges directed")
        .into_iter()
        .map(|(_, _, target)| target)
        .collect();

    assert_eq!(
        expected_edges, existing_edges,
        "confirm edges are there before deleting"
    );

    graph
        .mark_graph_seen(empty_change_set.vector_clock_id())
        .await
        .expect("Unable to mark empty graph as seen");

    let graph_with_deleted_edge = graph.real_clone().await;
    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;

    graph_with_deleted_edge
        .remove_edge(
            new_change_set,
            graph_with_deleted_edge
                .get_node_index_by_id(schema_id)
                .await
                .expect("Unable to get NodeIndex for schema"),
            schema_variant_2_index,
            EdgeWeightKindDiscriminants::Use,
        )
        .await
        .expect("Edge removal failed");

    let existing_edges: Vec<NodeIndex> = graph_with_deleted_edge
        .edges_directed(schema_id, Outgoing)
        .await
        .expect("able to get edges directed")
        .into_iter()
        .map(|(_, _, target)| target)
        .collect();

    assert_eq!(
        vec![schema_variant_index],
        existing_edges,
        "confirm edges after deletion"
    );

    graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    graph_with_deleted_edge
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    graph_with_deleted_edge
        .mark_graph_seen(new_change_set.vector_clock_id())
        .await
        .expect("Unable to mark new graph as seen");

    let (conflicts, updates) = graph
        .detect_conflicts_and_updates(
            empty_change_set.vector_clock_id(),
            &graph_with_deleted_edge,
            new_change_set.vector_clock_id(),
        )
        .await
        .expect("Failed to detect conflicts and updates");

    assert!(conflicts.is_empty());
    assert_eq!(1, updates.len());

    assert!(matches!(
        updates.first().expect("should be there"),
        Update::RemoveEdge { .. }
    ));
}

#[test]
async fn remove_unordered_node(ctx: &DalContext) {
    let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let change_set = &change_set;
    let graph = WorkspaceSnapshot::empty(ctx, change_set)
        .await
        .expect("Unable to create WorkspaceSnapshotGraph");
    let root_id = graph.root_id().await.expect("unable to get root id");

    let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let _schema_index = graph
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
        .await
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
        .await
        .expect("Unable to add schema variant");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
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
        .await
        .expect("Unable to add schema variant");

    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_2_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let expected_edges = HashSet::from([schema_variant_2_index, schema_variant_index]);

    let existing_edges: HashSet<NodeIndex> = graph
        .edges_directed(schema_id, Outgoing)
        .await
        .expect("unable to get edges directed")
        .into_iter()
        .map(|(_, _, target)| target)
        .collect();

    assert_eq!(
        expected_edges, existing_edges,
        "confirm edges are there before deleting"
    );

    graph
        .remove_edge_for_ulids(
            change_set,
            schema_id,
            schema_variant_2_id,
            EdgeWeightKindDiscriminants::Use,
        )
        .await
        .expect("Edge removal failed");

    let existing_edges: Vec<NodeIndex> = graph
        .edges_directed(schema_id, Outgoing)
        .await
        .expect("unable to get edges directed")
        .into_iter()
        .map(|(_, _, target)| target)
        .collect();

    assert_eq!(
        vec![schema_variant_index],
        existing_edges,
        "confirm edges after deletion"
    );
}

#[test]
async fn remove_ordered_node(ctx: &DalContext) {
    let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let change_set = &change_set;
    let graph = WorkspaceSnapshot::empty(ctx, change_set)
        .await
        .expect("should create snapshot");
    let root_id = graph.root_id().await.expect("should get root id");

    let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
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
        .await
        .expect("Unable to add schema");
    let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
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
        .await
        .expect("Unable to add schema variant");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    graph
        .add_edge(
            schema_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let func_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                func_id,
                ContentAddress::Func(ContentHash::new(FuncId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add func");
    graph
        .add_edge(
            root_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add root -> func edge");

    let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_ordered_node(
            change_set,
            NodeWeight::new_content(
                change_set,
                root_prop_id,
                ContentAddress::Prop(ContentHash::new(PropId::generate().to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add prop");
    graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            root_prop_id,
        )
        .await
        .expect("Unable to add schema variant -> prop edge");
    graph
        .add_edge(
            root_prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            func_id,
        )
        .await
        .expect("Unable to add prop -> func edge");
    graph.cleanup().await.expect("should clean up");
    // graph.dot();

    let ordered_prop_1_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_1_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            root_prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_1_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_1 edge");

    let ordered_prop_2_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    let ordered_prop_2_index = graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_2_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            root_prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_2_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_2 edge");

    let ordered_prop_3_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_3_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            root_prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_3_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_3 edge");

    let ordered_prop_4_id = change_set.generate_ulid().expect("Unable to generate Ulid");
    graph
        .add_node(
            NodeWeight::new_content(
                change_set,
                ordered_prop_4_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop");
    graph
        .add_ordered_edge(
            change_set,
            root_prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                .expect("Unable to create uses edge weight"),
            ordered_prop_4_id,
        )
        .await
        .expect("Unable to add prop -> ordered_prop_4 edge");

    graph.cleanup().await.expect("should clean up");
    // graph.dot();

    assert_eq!(
        vec![
            ordered_prop_1_id,
            ordered_prop_2_id,
            ordered_prop_3_id,
            ordered_prop_4_id,
        ],
        graph
            .ordered_children_for_node(root_prop_id,)
            .await
            .expect("Unable to find ordered children for node")
            .expect("Node is not an ordered node")
    );

    graph
        .remove_edge(
            change_set,
            graph
                .get_node_index_by_id(root_prop_id)
                .await
                .expect("Unable to get NodeIndex for prop"),
            ordered_prop_2_index,
            EdgeWeightKindDiscriminants::Use,
        )
        .await
        .expect("Unable to remove prop -> ordered_prop_2 edge");

    assert_eq!(
        vec![ordered_prop_1_id, ordered_prop_3_id, ordered_prop_4_id,],
        graph
            .ordered_children_for_node(root_prop_id,)
            .await
            .expect("Unable to find ordered children for node")
            .expect("Node is not an ordered node")
    );
    if let NodeWeight::Ordering(ordering_weight) = graph
        .get_node_weight(
            graph
                .get_node_index_by_id(
                    graph
                        .ordering_node_for_container(root_prop_id)
                        .await
                        .expect("Error getting ordering NodeIndex for prop")
                        .expect("Unable to find ordering NodeIndex")
                        .id(),
                )
                .await
                .expect("why am I doing this"),
        )
        .await
        .expect("Unable to get node weight")
        .as_ref()
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
async fn detect_conflicts_and_updates_simple_ordering_no_conflicts_no_updates_in_base(
    ctx: &DalContext,
) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let empty_change_set = &empty_change_set;
    let empty_graph = WorkspaceSnapshot::empty(ctx, empty_change_set)
        .await
        .expect("should create snapshot");
    let empty_root_id = empty_graph.root_id().await.expect("should get root id");

    let schema_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    empty_graph
        .add_edge(
            empty_root_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    empty_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let container_prop_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_ordered_node(
            empty_change_set,
            NodeWeight::new_content(
                empty_change_set,
                container_prop_id,
                ContentAddress::Prop(ContentHash::new(container_prop_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add container prop");
    empty_graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            container_prop_id,
        )
        .await
        .expect("Unable to add schema variant -> container prop edge");

    let ordered_prop_1_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_1_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 1");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_1_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 1 edge");

    let ordered_prop_2_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_2_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 2");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_2_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 2 edge");

    let ordered_prop_3_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_3_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 3");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_3_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 3 edge");

    let ordered_prop_4_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_4_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 4");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_4_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 4 edge");

    empty_graph.cleanup().await.expect("should clean up");
    // empty_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = empty_graph.real_clone().await;

    let ordered_prop_5_id = new_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    new_graph
        .add_node(
            NodeWeight::new_content(
                new_change_set,
                ordered_prop_5_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_5_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 5");
    new_graph
        .add_ordered_edge(
            new_change_set,
            container_prop_id,
            EdgeWeight::new(new_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_5_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 5 edge");

    new_graph.cleanup().await.expect("should clean up");
    // new_graph.dot();

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &empty_graph,
            empty_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert_eq!(Vec::<Conflict>::new(), conflicts);
    assert_eq!(Vec::<Update>::new(), updates);
}

#[test]
async fn detect_conflicts_and_updates_simple_ordering_no_conflicts_with_updates_in_base(
    ctx: &DalContext,
) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let empty_change_set = &empty_change_set;
    let empty_graph = WorkspaceSnapshot::empty(ctx, empty_change_set)
        .await
        .expect("should create snapshot");
    let empty_root_id = empty_graph.root_id().await.expect("should get root id");

    let schema_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    empty_graph
        .add_edge(
            empty_root_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    empty_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let container_prop_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_ordered_node(
            empty_change_set,
            NodeWeight::new_content(
                empty_change_set,
                container_prop_id,
                ContentAddress::Prop(ContentHash::new(container_prop_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add container prop");
    empty_graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            container_prop_id,
        )
        .await
        .expect("Unable to add schema variant -> container prop edge");

    let ordered_prop_1_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_1_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 1");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_1_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 1 edge");

    let ordered_prop_2_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_2_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 2");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_2_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 2 edge");

    let ordered_prop_3_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_3_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 3");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_3_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 3 edge");

    let ordered_prop_4_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_4_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 4");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_4_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 4 edge");

    // empty_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = empty_graph.real_clone().await;

    let ordered_prop_5_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_5_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_5_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 5");
    let new_edge_weight = EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
        .expect("Unable to create EdgeWeight");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            new_edge_weight.clone(),
            ordered_prop_5_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 5 edge");

    // new_graph.dot();

    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    empty_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &empty_graph,
            empty_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    let empty_graph_ordering_node = empty_graph
        .ordering_node_for_container(container_prop_id)
        .await
        .expect("Unable to get ordering node")
        .expect("No Ordering Node found");

    let new_graph_ordering_node = empty_graph
        .ordering_node_for_container(container_prop_id)
        .await
        .expect("Unable to get ordering node")
        .expect("No Ordering Node found");

    let ordinal_edge_weight = empty_graph
        .get_edges_between_nodes(new_graph_ordering_node.id(), ordered_prop_5_id)
        .await
        .expect("should not error when getting edge")
        .first()
        .expect("unable to get edge weight")
        .to_owned();

    assert_eq!(Vec::<Conflict>::new(), conflicts);
    assert_eq!(
        vec![
            Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(container_prop_id)
                    .await
                    .expect("Unable to get NodeIndex"),
                destination: empty_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .await
                    .expect("Unable to get NodeIndex"),
                edge_weight: new_edge_weight,
            },
            Update::ReplaceSubgraph {
                onto: empty_graph
                    .get_node_index_by_id(empty_graph_ordering_node.id())
                    .await
                    .expect("Ordering NodeIndex not found"),
                to_rebase: new_graph
                    .get_node_index_by_id(new_graph_ordering_node.id())
                    .await
                    .expect("Ordering NodeIndex not found"),
            },
            Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(new_graph_ordering_node.id())
                    .await
                    .expect("could not get node index by id"),
                destination: empty_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .await
                    .expect("could not get node index by id"),
                edge_weight: ordinal_edge_weight,
            }
        ],
        updates
    );
}

#[test]
async fn detect_conflicts_and_updates_simple_ordering_with_conflicting_ordering_updates(
    ctx: &DalContext,
) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let empty_change_set = &empty_change_set;
    let empty_graph = WorkspaceSnapshot::empty(ctx, empty_change_set)
        .await
        .expect("Unable to create WorkspaceSnapshotGraph");
    let empty_root_id = empty_graph.root_id().await.expect("Unable to get root id");

    let schema_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    empty_graph
        .add_edge(
            empty_root_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    empty_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let container_prop_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_ordered_node(
            empty_change_set,
            NodeWeight::new_content(
                empty_change_set,
                container_prop_id,
                ContentAddress::Prop(ContentHash::new(container_prop_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add container prop");
    empty_graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            container_prop_id,
        )
        .await
        .expect("Unable to add schema variant -> container prop edge");

    let ordered_prop_1_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_1_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 1");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_1_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 1 edge");

    let ordered_prop_2_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_2_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 2");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_2_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 2 edge");

    let ordered_prop_3_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_3_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 3");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_3_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 3 edge");

    let ordered_prop_4_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_4_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 4");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_4_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 4 edge");

    // empty_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = empty_graph.real_clone().await;

    let new_order = vec![
        ordered_prop_2_id,
        ordered_prop_1_id,
        ordered_prop_4_id,
        ordered_prop_3_id,
    ];
    new_graph
        .update_order(new_change_set, container_prop_id, new_order)
        .await
        .expect("Unable to update order of container prop's children");

    let ordered_prop_5_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_5_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_5_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 5");
    let new_edge_weight = EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
        .expect("Unable to create EdgeWeight");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            new_edge_weight.clone(),
            ordered_prop_5_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 5 edge");
    // new_graph.dot();

    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    empty_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &empty_graph,
            empty_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    let empty_graph_ordering_node = empty_graph
        .ordering_node_for_container(container_prop_id)
        .await
        .expect("Unable to get ordering node")
        .expect("No Ordering Node found");

    let new_graph_ordering_node = empty_graph
        .ordering_node_for_container(container_prop_id)
        .await
        .expect("Unable to get ordering node")
        .expect("No Ordering Node found");

    let ordinal_edge_weight = empty_graph
        .get_edges_between_nodes(new_graph_ordering_node.id(), ordered_prop_5_id)
        .await
        .expect("should not error when getting edge")
        .first()
        .expect("unable to get edge weight")
        .to_owned();

    assert_eq!(
        vec![Conflict::ChildOrder {
            onto: empty_graph
                .get_node_index_by_id(empty_graph_ordering_node.id())
                .await
                .expect("Ordering NodeIndex not found"),
            to_rebase: new_graph
                .get_node_index_by_id(new_graph_ordering_node.id())
                .await
                .expect("Ordering NodeIndex not found"),
        }],
        conflicts
    );
    assert_eq!(
        vec![
            Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(container_prop_id)
                    .await
                    .expect("Unable to get new_graph container NodeIndex"),
                destination: empty_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .await
                    .expect("Unable to get ordered prop 5 NodeIndex"),
                edge_weight: new_edge_weight,
            },
            Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(new_graph_ordering_node.id())
                    .await
                    .expect("could not get node index by id"),
                destination: empty_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .await
                    .expect("could not get node index by id"),
                edge_weight: ordinal_edge_weight,
            }
        ],
        updates
    );
}

#[test]
async fn detect_conflicts_and_updates_simple_ordering_with_no_conflicts_add_in_onto_remove_in_to_rebase(
    ctx: &DalContext,
) {
    let empty_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let empty_change_set = &empty_change_set;
    let empty_graph = WorkspaceSnapshot::empty(ctx, empty_change_set)
        .await
        .expect("should create snapshot");
    let empty_root_id = empty_graph.root_id().await.expect("should get root id");

    let schema_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_id,
                ContentAddress::Schema(ContentHash::from("Schema A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema A");
    let schema_variant_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                schema_variant_id,
                ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add Schema Variant A");

    empty_graph
        .add_edge(
            empty_root_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_id,
        )
        .await
        .expect("Unable to add root -> schema edge");
    empty_graph
        .add_edge(
            schema_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            schema_variant_id,
        )
        .await
        .expect("Unable to add schema -> schema variant edge");

    let container_prop_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_ordered_node(
            empty_change_set,
            NodeWeight::new_content(
                empty_change_set,
                container_prop_id,
                ContentAddress::Prop(ContentHash::new(container_prop_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add container prop");
    empty_graph
        .add_edge(
            schema_variant_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            container_prop_id,
        )
        .await
        .expect("Unable to add schema variant -> container prop edge");

    let ordered_prop_1_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_1_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_1_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 1");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_1_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 1 edge");

    let ordered_prop_2_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_2_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_2_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 2");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_2_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 2 edge");

    let ordered_prop_3_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_3_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_3_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 3");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_3_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 3 edge");

    let ordered_prop_4_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_4_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_4_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 4");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            ordered_prop_4_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 4 edge");

    empty_graph.cleanup().await.expect("should clean up");
    empty_graph
        .mark_graph_seen(empty_change_set.vector_clock_id())
        .await
        .expect("Unable to update recently seen information");
    // empty_graph.dot();

    let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let new_change_set = &new_change_set;
    let new_graph = empty_graph.real_clone().await;

    new_graph
        .remove_edge(
            new_change_set,
            empty_graph
                .get_node_index_by_id(container_prop_id)
                .await
                .expect("Unable to get NodeIndex"),
            empty_graph
                .get_node_index_by_id(ordered_prop_2_id)
                .await
                .expect("Unable to get NodeIndex"),
            EdgeWeightKindDiscriminants::Use,
        )
        .await
        .expect("Unable to remove container prop -> prop 2 edge");

    let ordered_prop_5_id = empty_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    empty_graph
        .add_node(
            NodeWeight::new_content(
                empty_change_set,
                ordered_prop_5_id,
                ContentAddress::Prop(ContentHash::new(ordered_prop_5_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add ordered prop 5");

    let new_edge_weight = EdgeWeight::new(empty_change_set, EdgeWeightKind::new_use())
        .expect("Unable to create EdgeWeight");
    empty_graph
        .add_ordered_edge(
            empty_change_set,
            container_prop_id,
            new_edge_weight.clone(),
            ordered_prop_5_id,
        )
        .await
        .expect("Unable to add container prop -> ordered prop 5 edge");
    let ordering_node = empty_graph
        .ordering_node_for_container(container_prop_id)
        .await
        .expect("Unable to get ordering node")
        .expect("No Ordering Node found");
    let ordinal_edge_weight = empty_graph
        .get_edges_between_nodes(ordering_node.id(), ordered_prop_5_id)
        .await
        .expect("should not error when getting edge")
        .first()
        .expect("unable to get edge weight")
        .to_owned();

    empty_graph.cleanup().await.expect("should clean up");
    // empty_graph.dot();

    new_graph.cleanup().await.expect("should clean up");
    // new_graph.dot();

    new_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");
    empty_graph
        .calculate_entire_merkle_tree_hash()
        .await
        .expect("calculate mth");

    let (conflicts, updates) = new_graph
        .detect_conflicts_and_updates(
            new_change_set.vector_clock_id(),
            &empty_graph,
            empty_change_set.vector_clock_id(),
        )
        .await
        .expect("Unable to detect conflicts and updates");

    assert_eq!(Vec::<Conflict>::new(), conflicts);
    assert_eq!(
        vec![
            Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(container_prop_id)
                    .await
                    .expect("Unable to get node index"),
                destination: empty_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .await
                    .expect("Unable to get ordered prop 5 NodeIndex"),
                edge_weight: new_edge_weight,
            },
            Update::NewEdge {
                source: new_graph
                    .get_node_index_by_id(ordering_node.id())
                    .await
                    .expect("could not get node index by id"),
                destination: empty_graph
                    .get_node_index_by_id(ordered_prop_5_id)
                    .await
                    .expect("could not get node index by id"),
                edge_weight: ordinal_edge_weight,
            }
        ],
        updates
    );
}

#[test]
async fn add_ordered_node_below_root(ctx: &DalContext) {
    let base_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let active_change_set = &base_change_set;
    let graph = WorkspaceSnapshot::empty(ctx, active_change_set)
        .await
        .expect("should create snapshot");
    let root_id = graph.root_id().await.expect("should get root id");

    let prop_id = active_change_set
        .generate_ulid()
        .expect("Unable to generate Ulid");
    graph
        .add_ordered_node(
            active_change_set,
            NodeWeight::new_content(
                active_change_set,
                prop_id,
                ContentAddress::Prop(ContentHash::new(prop_id.to_string().as_bytes())),
            )
            .expect("Unable to create NodeWeight"),
        )
        .await
        .expect("Unable to add prop");

    graph
        .add_edge(
            root_id,
            EdgeWeight::new(active_change_set, EdgeWeightKind::new_use())
                .expect("Unable to create EdgeWeight"),
            prop_id,
        )
        .await
        .expect("Unable to add root -> prop edge");

    graph.cleanup().await.expect("unable to clean graph");
    assert_eq!(
        Vec::<Ulid>::new(),
        graph
            .ordered_children_for_node(prop_id,)
            .await
            .expect("Unable to find ordered children for node")
            .expect("Node is not an ordered node")
    );
}
