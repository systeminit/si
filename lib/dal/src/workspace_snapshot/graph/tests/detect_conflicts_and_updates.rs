#[allow(clippy::panic)]
#[cfg(test)]
mod test {
    use petgraph::prelude::EdgeRef;
    use petgraph::Outgoing;
    use pretty_assertions_sorted::assert_eq;
    use si_events::ContentHash;
    use std::collections::HashMap;

    use crate::change_set::ChangeSet;
    use crate::workspace_snapshot::conflict::Conflict;
    use crate::workspace_snapshot::content_address::ContentAddress;
    use crate::workspace_snapshot::edge_weight::{
        EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
    };
    use crate::workspace_snapshot::graph::tests::{add_edges, add_prop_nodes_to_graph};
    use crate::workspace_snapshot::node_weight::NodeWeight;
    use crate::workspace_snapshot::update::Update;
    use crate::{PropKind, WorkspaceSnapshotGraph};

    #[derive(Debug, PartialEq)]
    struct ConflictsAndUpdates {
        conflicts: Vec<Conflict>,
        updates: Vec<Update>,
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
        new_graph.dot();

        base_graph
            .update_content(
                base_change_set,
                component_id,
                ContentHash::from("Base Updated Component A"),
            )
            .expect("Unable to update Component A");

        base_graph.cleanup();
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
        base_graph.dot();

        new_graph
            .update_content(
                new_change_set,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
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

        base_graph.dot();
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

    #[test]
    fn detect_conflicts_and_updates_single_removal_update() {
        let nodes = ["a", "b", "c"];
        let edges = [(None, "a"), (None, "b"), (Some("a"), "c"), (Some("c"), "b")];

        let base_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let base_change_set = &base_change_set;
        let mut base_graph = WorkspaceSnapshotGraph::new(base_change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Add all nodes from the slice and store their references in a hash map.
        let mut node_id_map = HashMap::new();
        for node in nodes {
            // "props" here are just nodes that are easy to create and render the name on the dot
            // output. there is no domain modeling in this test.
            let node_id = base_change_set
                .generate_ulid()
                .expect("Unable to generate Ulid");
            let prop_node_weight = NodeWeight::new_prop(
                base_change_set,
                node_id,
                PropKind::Object,
                node,
                ContentHash::new(node.as_bytes()),
            )
            .expect("create prop node weight");
            base_graph
                .add_node(prop_node_weight)
                .expect("Unable to add prop");

            node_id_map.insert(node, node_id);
        }

        // Add all edges from the slice.
        for (source, target) in edges {
            let source = match source {
                None => base_graph.root_index,
                Some(node) => base_graph
                    .get_node_index_by_id(
                        node_id_map
                            .get(node)
                            .copied()
                            .expect("source node should have an id"),
                    )
                    .expect("get node index by id"),
            };

            let target = base_graph
                .get_node_index_by_id(
                    node_id_map
                        .get(target)
                        .copied()
                        .expect("target node should have an id"),
                )
                .expect("get node index by id");

            base_graph
                .add_edge(
                    source,
                    EdgeWeight::new(base_change_set, EdgeWeightKind::new_use())
                        .expect("create edge weight"),
                    target,
                )
                .expect("add edge");
        }

        // Clean up the graph before ensuring that it was constructed properly.
        base_graph.cleanup();

        // Ensure the graph construction worked.
        for (source, target) in edges {
            let source_idx = match source {
                None => base_graph.root_index,
                Some(node) => base_graph
                    .get_node_index_by_id(
                        node_id_map
                            .get(node)
                            .copied()
                            .expect("source node should have an id"),
                    )
                    .expect("get node index by id"),
            };

            let target_idx = base_graph
                .get_node_index_by_id(
                    node_id_map
                        .get(target)
                        .copied()
                        .expect("target node should have an id"),
                )
                .expect("get node index by id");

            assert!(
                base_graph
                    .edges_directed(source_idx, Outgoing)
                    .any(|edge_ref| edge_ref.target() == target_idx),
                "An edge from {} to {} should exist",
                source.unwrap_or("root"),
                target
            );
        }
        for (_, id) in node_id_map.iter() {
            let idx_for_node = base_graph
                .get_node_index_by_id(*id)
                .expect("able to get idx by id");
            base_graph
                .get_node_weight(idx_for_node)
                .expect("node with weight in graph");
        }

        // Cache all IDs for later.
        let a_id = *node_id_map.get("a").expect("could not get node id");
        let b_id = *node_id_map.get("b").expect("could not get node id");
        let c_id = *node_id_map.get("c").expect("could not get node id");

        // Prepare the graph for "forking" and fork it. Create a new change set after.
        base_graph
            .mark_graph_seen(base_change_set.vector_clock_id())
            .expect("could not mark as seen");
        let mut new_graph = base_graph.clone();
        let new_change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let new_change_set = &new_change_set;

        // Remove the first edge involving "c".
        let a_idx = new_graph
            .get_node_index_by_id(a_id)
            .expect("could not get node index by id");
        let c_idx = new_graph
            .get_node_index_by_id(c_id)
            .expect("could not get node index by id");
        new_graph
            .remove_edge(
                new_change_set,
                a_idx,
                c_idx,
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("could not remove edge");

        // Remove the second edge involving "c".
        let b_idx = new_graph
            .get_node_index_by_id(b_id)
            .expect("could not get node index by id");
        let c_idx = new_graph
            .get_node_index_by_id(c_id)
            .expect("could not get node index by id");
        new_graph
            .remove_edge(
                new_change_set,
                c_idx,
                b_idx,
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("could not remove edge");

        // Perform the removal
        new_graph.remove_node(c_idx);
        new_graph.remove_node_id(c_id);

        // Prepare for conflicts and updates detection
        new_graph
            .mark_graph_seen(new_change_set.vector_clock_id())
            .expect("could not mark graph seen");
        new_graph.cleanup();

        let (conflicts, updates) = base_graph
            .detect_conflicts_and_updates(
                base_change_set.vector_clock_id(),
                &new_graph,
                new_change_set.vector_clock_id(),
            )
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts.is_empty());
        assert_eq!(
            vec![Update::RemoveEdge {
                source: a_idx,
                destination: c_idx,
                edge_kind: EdgeWeightKindDiscriminants::Use,
            }],
            updates
        );
    }

    #[test]
    fn detect_exclusive_edge_conflict() {
        let base_nodes = ["r", "q"];
        let base_edges = [(None, "r"), (Some("r"), "q")];

        let base_change_set = ChangeSet::new_local().expect("unable to create change set");
        let base_change_set = &base_change_set;
        let mut base_graph =
            WorkspaceSnapshotGraph::new(base_change_set).expect("unable to make base graph");

        let mut node_id_map =
            add_prop_nodes_to_graph(&mut base_graph, base_change_set, &base_nodes);
        add_edges(&mut base_graph, &node_id_map, base_change_set, &base_edges);
        base_graph.cleanup();
        base_graph
            .mark_graph_seen(base_change_set.vector_clock_id())
            .expect("unable to mark seen");

        let a_new_nodes = ["a"];
        let a_edges = [(Some("q"), "a")];

        let change_set_a = ChangeSet::new_local().expect("unable to create change set");
        let change_set_a = &change_set_a;
        let mut graph_a = base_graph.clone();
        node_id_map.extend(add_prop_nodes_to_graph(
            &mut graph_a,
            change_set_a,
            &a_new_nodes,
        ));
        add_edges(&mut graph_a, &node_id_map, change_set_a, &a_edges);
        graph_a.cleanup();
        graph_a
            .mark_graph_seen(change_set_a.vector_clock_id())
            .expect("unable to mark seen");

        let b_new_nodes = ["b"];
        let b_edges = [(Some("q"), "b")];

        let change_set_b = ChangeSet::new_local().expect("unable to create change set");
        let change_set_b = &change_set_b;
        let mut graph_b = base_graph.clone();
        node_id_map.extend(add_prop_nodes_to_graph(
            &mut graph_b,
            change_set_b,
            &b_new_nodes,
        ));
        add_edges(&mut graph_b, &node_id_map, change_set_b, &b_edges);
        graph_b.cleanup();
        graph_b
            .mark_graph_seen(change_set_b.vector_clock_id())
            .expect("unable to mark seen");

        let (conflicts, _) = graph_a
            .detect_conflicts_and_updates(
                change_set_a.vector_clock_id(),
                &graph_b,
                change_set_b.vector_clock_id(),
            )
            .expect("able to detect conflicts and updates");

        let a_q_node_idx = graph_a
            .get_node_index_by_id(*node_id_map.get("q").expect("should have an id for 'q'"))
            .expect("able to get q node index");
        let b_q_node_idx = graph_b
            .get_node_index_by_id(*node_id_map.get("q").expect("should have an id for 'q'"))
            .expect("able to get q node index");
        let a_node_idx = graph_a
            .get_node_index_by_id(*node_id_map.get("a").expect("should have an id for 'a'"))
            .expect("able to get a node index");
        let b_node_idx = graph_b
            .get_node_index_by_id(*node_id_map.get("b").expect("should have an id for 'b'"))
            .expect("able to get b node index");

        assert_eq!(
            vec![Conflict::ExclusiveEdgeMismatch {
                source: a_q_node_idx,
                destination: a_node_idx,
                edge_kind: EdgeWeightKindDiscriminants::Use,
            }],
            conflicts
        );

        let (conflicts, _) = graph_b
            .detect_conflicts_and_updates(
                change_set_b.vector_clock_id(),
                &graph_a,
                change_set_a.vector_clock_id(),
            )
            .expect("able to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::ExclusiveEdgeMismatch {
                source: b_q_node_idx,
                destination: b_node_idx,
                edge_kind: EdgeWeightKindDiscriminants::Use,
            }],
            conflicts
        );
    }
}
