use dal::workspace_snapshot::update::Update;
use pretty_assertions_sorted::assert_eq;
use si_events::*;

use dal::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use dal::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use dal::workspace_snapshot::node_weight::NodeWeight;
use dal::workspace_snapshot::node_weight::{ContentNodeWeight, FuncNodeWeight};
use dal::DalContext;
use dal::{change_set::ChangeSet, workspace_snapshot::content_address::ContentAddress};
use dal::{func::FuncKind, WorkspaceSnapshot};
use dal_test::test;

#[test]
async fn simulate_rebase(ctx: &DalContext) {
    let to_rebase_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let to_rebase_change_set = &to_rebase_change_set;
    let to_rebase = WorkspaceSnapshot::initial(ctx, to_rebase_change_set)
        .await
        .expect("Unable to create WorkspaceSnapshotGraph");

    // Set up the to rebase graph.
    let func_category_node_id = to_rebase
        .get_category_node(None, CategoryNodeKind::Func)
        .await
        .expect("should get func categopry node");

    let schema_category_node_id = to_rebase
        .get_category_node(None, CategoryNodeKind::Func)
        .await
        .expect("should get func categopry node");

    // Create the onto graph from the to rebase graph.
    let onto_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
    let onto_change_set = &onto_change_set;
    let onto = to_rebase.real_clone().await;

    // FuncCategory --Use--> Func
    let func_id = onto_change_set
        .generate_ulid()
        .expect("could not generate ulid");
    let func_node_weight = FuncNodeWeight::new(
        onto_change_set,
        func_id,
        ContentAddress::Func(ContentHash::from("foo")),
        "foo".to_string(),
        FuncKind::Intrinsic,
    )
    .expect("could not create func node weight");
    onto.add_node(NodeWeight::Func(func_node_weight))
        .await
        .expect("could not add node");
    onto.add_edge(
        func_category_node_id,
        EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
            .expect("could not create edge weight"),
        func_id,
    )
    .await
    .expect("could not add edge");

    // SchemaCategory --Use--> Schema
    let schema_id = onto_change_set
        .generate_ulid()
        .expect("could not generate ulid");
    let schema_node_weight = ContentNodeWeight::new(
        onto_change_set,
        schema_id,
        ContentAddress::Schema(ContentHash::from("foo")),
    )
    .expect("could not create func node weight");
    onto.add_node(NodeWeight::Content(schema_node_weight))
        .await
        .expect("could not add node");
    onto.add_edge(
        schema_category_node_id,
        EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
            .expect("could not create edge weight"),
        schema_id,
    )
    .await
    .expect("could not add edge");

    // Schema --Use--> SchemaVariant
    let schema_variant_node_id = onto_change_set
        .generate_ulid()
        .expect("could not generate ulid");
    let schema_variant_node_weight = ContentNodeWeight::new(
        onto_change_set,
        schema_variant_node_id,
        ContentAddress::SchemaVariant(ContentHash::from("foo")),
    )
    .expect("could not create func node weight");
    onto.add_node(NodeWeight::Content(schema_variant_node_weight))
        .await
        .expect("could not add node");
    onto.add_edge(
        schema_id,
        EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
            .expect("could not create edge weight"),
        schema_variant_node_id,
    )
    .await
    .expect("could not add edge");

    // SchemaVariant --Use--> Func
    onto.get_node_index_by_id(func_id)
        .await
        .expect("could not get node index by id");
    onto.add_edge(
        schema_variant_node_id,
        EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
            .expect("could not create edge weight"),
        func_id,
    )
    .await
    .expect("could not add edge");

    // Before cleanup, detect conflicts and updates.
    let (before_cleanup_conflicts, mut before_cleanup_updates) = to_rebase
        .detect_conflicts_and_updates(
            to_rebase_change_set.vector_clock_id(),
            &onto,
            onto_change_set.vector_clock_id(),
        )
        .await
        .expect("could not detect conflicts and updates");

    // Cleanup and check node count.
    onto.cleanup().await.expect("should clean up");
    to_rebase.cleanup().await.expect("should clean up");
    assert_eq!(
        9,                       // expected
        onto.node_count().await  // actual
    );

    // Detect conflicts and updates. Ensure cleanup did not affect the results.
    let (conflicts, mut updates) = to_rebase
        .detect_conflicts_and_updates(
            to_rebase_change_set.vector_clock_id(),
            &onto,
            onto_change_set.vector_clock_id(),
        )
        .await
        .expect("could not detect conflicts and updates");

    assert!(conflicts.is_empty());
    assert_eq!(
        2,             // expected
        updates.len()  // actual
    );
    assert_eq!(
        before_cleanup_conflicts, // expected
        conflicts                 // actual
    );

    // detect_conflicts_and_updates is not deterministic about order, so we need to sort before
    // comparing
    let match_update = |k: &Update| match k {
        Update::NewEdge {
            source,
            destination,
            ..
        } => (*source, *destination),
        Update::RemoveEdge { .. } => todo!(),
        Update::ReplaceSubgraph { .. } => todo!(),
    };
    before_cleanup_updates.sort_by_key(match_update);
    updates.sort_by_key(match_update);
    assert_eq!(
        before_cleanup_updates, // expected
        updates                 // actual
    );

    // Ensure that we do not have duplicate updates.
    let mut deduped_updates = updates.clone();
    deduped_updates.dedup();
    assert_eq!(
        deduped_updates.len(), // expected
        updates.len()          // actual
    );

    // Perform the updates. In the future, we may want to see if the onto and resulting to
    // rebase graphs are logically equivalent after updates are performed.
    to_rebase
        .perform_updates(to_rebase_change_set, &onto, &updates)
        .await
        .expect("could not perform updates");
}
