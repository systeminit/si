#[allow(clippy::panic)]
#[cfg(test)]
mod test {
    use pretty_assertions_sorted::assert_eq;
    use si_events::{
        ContentHash,
        ulid::Ulid,
    };

    use crate::{
        WorkspaceSnapshotGraphVCurrent,
        func::FuncKind,
        workspace_snapshot::{
            content_address::ContentAddress,
            edge_weight::{
                EdgeWeight,
                EdgeWeightKind,
            },
            node_weight::{
                ContentNodeWeight,
                FuncNodeWeight,
                NodeWeight,
                category_node_weight::CategoryNodeKind,
            },
        },
    };

    #[test]
    fn simulate_rebase() {
        let mut to_rebase = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Set up the to rebase graph.
        let schema_category_node_index = to_rebase
            .add_category_node(Ulid::new(), Ulid::new(), CategoryNodeKind::Schema)
            .expect("could not add category node");
        to_rebase
            .add_edge(
                to_rebase.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                schema_category_node_index,
            )
            .expect("could not add edge");
        let func_category_node_index = to_rebase
            .add_category_node(Ulid::new(), Ulid::new(), CategoryNodeKind::Func)
            .expect("could not add category node");
        to_rebase
            .add_edge(
                to_rebase.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                func_category_node_index,
            )
            .expect("could not add edge");

        // Create the onto graph from the to rebase graph.
        let mut onto = to_rebase.clone();

        // FuncCategory --Use--> Func
        let func_id = onto.generate_ulid().expect("could not generate ulid");
        let func_node_weight = FuncNodeWeight::new(
            func_id,
            Ulid::new(),
            ContentAddress::Func(ContentHash::from("foo")),
            "foo".to_string(),
            FuncKind::Intrinsic,
        );

        let func_node_index = onto
            .add_or_replace_node(NodeWeight::Func(func_node_weight))
            .expect("could not add node");
        onto.add_edge(
            func_category_node_index,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            func_node_index,
        )
        .expect("could not add edge");

        // SchemaCategory --Use--> Schema
        let schema_node_weight = ContentNodeWeight::new(
            onto.generate_ulid().expect("could not generate ulid"),
            Ulid::new(),
            ContentAddress::Schema(ContentHash::from("foo")),
        );

        let schema_node_index = onto
            .add_or_replace_node(NodeWeight::Content(schema_node_weight))
            .expect("could not add node");
        onto.add_edge(
            schema_category_node_index,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            schema_node_index,
        )
        .expect("could not add edge");

        // Schema --Use--> SchemaVariant
        let schema_variant_node_weight = ContentNodeWeight::new(
            onto.generate_ulid().expect("could not generate ulid"),
            Ulid::new(),
            ContentAddress::SchemaVariant(ContentHash::from("foo")),
        );

        let schema_variant_node_index = onto
            .add_or_replace_node(NodeWeight::Content(schema_variant_node_weight))
            .expect("could not add node");
        onto.add_edge(
            schema_node_index,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            schema_variant_node_index,
        )
        .expect("could not add edge");

        // SchemaVariant --Use--> Func
        let func_node_index = onto
            .get_node_index_by_id(func_id)
            .expect("could not get node index by id");
        onto.add_edge(
            schema_variant_node_index,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            func_node_index,
        )
        .expect("could not add edge");

        // Cleanup and check node count.
        onto.cleanup_and_merkle_tree_hash().expect("merkle it!");
        to_rebase
            .cleanup_and_merkle_tree_hash()
            .expect("merkle it!");
        assert_eq!(
            19,                // expected
            onto.node_count()  // actual
        );

        // Detect conflicts and updates. Ensure cleanup did not affect the results.
        let updates = to_rebase.detect_updates(&onto);
        assert_eq!(
            7,             // expected
            updates.len()  // actual
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
            .perform_updates(&updates)
            .expect("could not perform updates");
    }
}
