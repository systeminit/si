#[allow(clippy::panic)]
#[cfg(test)]
mod test {
    use pretty_assertions_sorted::assert_eq;
    use si_events::ContentHash;

    use crate::change_set::ChangeSet;
    use crate::func::FuncKind;
    use crate::workspace_snapshot::content_address::ContentAddress;
    use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
    use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
    use crate::workspace_snapshot::node_weight::NodeWeight;
    use crate::workspace_snapshot::node_weight::{ContentNodeWeight, FuncNodeWeight};
    use crate::WorkspaceSnapshotGraph;

    #[test]
    fn simulate_rebase() {
        let to_rebase_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let to_rebase_change_set = &to_rebase_change_set;
        let mut to_rebase = WorkspaceSnapshotGraph::new(to_rebase_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Set up the to rebase graph.
        let schema_category_node_index = to_rebase
            .add_category_node(to_rebase_change_set, CategoryNodeKind::Schema)
            .expect("could not add category node");
        to_rebase
            .add_edge(
                to_rebase.root_index,
                EdgeWeight::new(to_rebase_change_set, EdgeWeightKind::new_use())
                    .expect("could not create edge weight"),
                schema_category_node_index,
            )
            .expect("could not add edge");
        let func_category_node_index = to_rebase
            .add_category_node(to_rebase_change_set, CategoryNodeKind::Func)
            .expect("could not add category node");
        to_rebase
            .add_edge(
                to_rebase.root_index,
                EdgeWeight::new(to_rebase_change_set, EdgeWeightKind::new_use())
                    .expect("could not create edge weight"),
                func_category_node_index,
            )
            .expect("could not add edge");

        // Create the onto graph from the to rebase graph.
        let onto_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let onto_change_set = &onto_change_set;
        let mut onto = to_rebase.clone();

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
        let func_node_index = onto
            .add_node(NodeWeight::Func(func_node_weight))
            .expect("could not add node");
        onto.add_edge(
            func_category_node_index,
            EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
                .expect("could not create edge weight"),
            func_node_index,
        )
        .expect("could not add edge");

        // SchemaCategory --Use--> Schema
        let schema_node_weight = ContentNodeWeight::new(
            onto_change_set,
            onto_change_set
                .generate_ulid()
                .expect("could not generate ulid"),
            ContentAddress::Schema(ContentHash::from("foo")),
        )
        .expect("could not create func node weight");
        let schema_node_index = onto
            .add_node(NodeWeight::Content(schema_node_weight))
            .expect("could not add node");
        onto.add_edge(
            schema_category_node_index,
            EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
                .expect("could not create edge weight"),
            schema_node_index,
        )
        .expect("could not add edge");

        // Schema --Use--> SchemaVariant
        let schema_variant_node_weight = ContentNodeWeight::new(
            onto_change_set,
            onto_change_set
                .generate_ulid()
                .expect("could not generate ulid"),
            ContentAddress::SchemaVariant(ContentHash::from("foo")),
        )
        .expect("could not create func node weight");
        let schema_variant_node_index = onto
            .add_node(NodeWeight::Content(schema_variant_node_weight))
            .expect("could not add node");
        onto.add_edge(
            schema_node_index,
            EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
                .expect("could not create edge weight"),
            schema_variant_node_index,
        )
        .expect("could not add edge");

        // SchemaVariant --Use--> Func
        let func_node_index = onto
            .get_node_index_by_id(func_id)
            .expect("could not get node index by id");
        onto.add_edge(
            schema_variant_node_index,
            EdgeWeight::new(onto_change_set, EdgeWeightKind::new_use())
                .expect("could not create edge weight"),
            func_node_index,
        )
        .expect("could not add edge");

        // Before cleanup, detect conflicts and updates.
        let (before_cleanup_conflicts, before_cleanup_updates) = to_rebase
            .detect_conflicts_and_updates(
                to_rebase_change_set.vector_clock_id(),
                &onto,
                onto_change_set.vector_clock_id(),
            )
            .expect("could not detect conflicts and updates");

        // Cleanup and check node count.
        onto.cleanup();
        to_rebase.cleanup();
        assert_eq!(
            6,                 // expected
            onto.node_count()  // actual
        );

        // Detect conflicts and updates. Ensure cleanup did not affect the results.
        let (conflicts, updates) = to_rebase
            .detect_conflicts_and_updates(
                to_rebase_change_set.vector_clock_id(),
                &onto,
                onto_change_set.vector_clock_id(),
            )
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
            .expect("could not perform updates");
    }
}
