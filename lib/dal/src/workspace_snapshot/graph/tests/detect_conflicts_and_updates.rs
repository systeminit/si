#[allow(clippy::panic)]
#[cfg(test)]
mod test {
    use petgraph::prelude::*;
    use petgraph::Outgoing;
    use pretty_assertions_sorted::assert_eq;
    use si_events::ulid::Ulid;
    use si_events::ContentHash;
    use si_events::VectorClockId;
    use std::collections::HashMap;

    use crate::workspace_snapshot::content_address::ContentAddress;
    use crate::workspace_snapshot::edge_weight::{
        EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
    };
    use crate::workspace_snapshot::graph::tests::add_prop_nodes_to_graph;
    use crate::workspace_snapshot::node_weight::NodeWeight;
    use crate::workspace_snapshot::update::Update;
    use crate::workspace_snapshot::{
        conflict::Conflict, graph::ConflictsAndUpdates, NodeInformation,
    };
    use crate::NodeWeightDiscriminants;
    use crate::{PropKind, WorkspaceSnapshotGraphV1};

    fn get_root_node_info(graph: &WorkspaceSnapshotGraphV1) -> NodeInformation {
        let root_id = graph
            .get_node_weight(graph.root_index)
            .expect("Unable to get root node")
            .id();

        NodeInformation {
            node_weight_kind: NodeWeightDiscriminants::Content,
            id: root_id.into(),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_no_updates_in_base() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut initial_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_variant_id,
                    Ulid::new(),
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = initial_graph.clone();

        let component_id = new_graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_vector_clock_id,
                    component_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        new_graph
            .add_edge(
                new_graph.root_index,
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(
                new_vector_clock_id,
                &initial_graph,
                initial_vector_clock_id,
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            ConflictsAndUpdates {
                conflicts: Vec::new(),
                updates: Vec::new()
            },
            conflicts_and_updates
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_purely_new_content_in_base() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);

        let mut base_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_variant_id,
                    Ulid::new(),
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = base_graph.clone();

        let new_onto_component_id = new_graph.generate_ulid().expect("Unable to generate Ulid");
        let new_onto_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    new_onto_component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        let _new_onto_root_component_edge_index = base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.dot();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(new_vector_clock_id, &base_graph, initial_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts_and_updates.conflicts.is_empty());

        let _new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match conflicts_and_updates.updates.as_slice() {
            [Update::NewNode { .. }, Update::NewEdge { edge_weight, .. }, Update::NewEdge { .. }] =>
            {
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_with_purely_new_content_in_new_graph() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);

        let mut base_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let component_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = base_graph.clone();

        let new_component_id = new_graph.generate_ulid().expect("Unable to generate Ulid");
        let new_component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_vector_clock_id,
                    new_component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        new_graph
            .add_edge(
                new_graph.root_index,
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_component_index,
            )
            .expect("Unable to add root -> component edge");

        new_graph.cleanup();
        new_graph.dot();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(new_vector_clock_id, &base_graph, initial_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts_and_updates.updates.is_empty());
        assert!(conflicts_and_updates.conflicts.is_empty());

        let conflicts_and_updates = base_graph
            .detect_conflicts_and_updates(initial_vector_clock_id, &new_graph, new_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts_and_updates.conflicts.is_empty());

        match conflicts_and_updates.updates.as_slice() {
            [Update::NewNode { .. }, Update::NewEdge { edge_weight, .. }] => {
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_updates_on_both_sides() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_variant_id,
                    Ulid::new(),
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = base_graph.clone();

        let component_id = new_graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_vector_clock_id,
                    component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        new_graph
            .add_edge(
                new_graph.root_index,
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        new_graph
            .add_edge(
                new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        new_graph.dot();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");

        let new_onto_component_id = new_graph.generate_ulid().expect("Unable to generate Ulid");
        let new_onto_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    new_onto_component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component B")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component B");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                new_onto_component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(new_onto_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(new_vector_clock_id, &base_graph, initial_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts_and_updates.conflicts.is_empty());

        let _new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match conflicts_and_updates.updates.as_slice() {
            [Update::NewNode { .. }, Update::NewEdge { edge_weight, .. }, Update::NewEdge { .. }] =>
            {
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_content_conflict() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_variant_id,
                    Ulid::new(),
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("mark graph seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = base_graph.clone();

        new_graph
            .update_content(
                new_vector_clock_id,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
        new_graph.dot();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("mark graph seen");

        base_graph
            .update_content(
                initial_vector_clock_id,
                component_id,
                ContentHash::from("Base Updated Component A"),
            )
            .expect("Unable to update Component A");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("mark graph seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(new_vector_clock_id, &base_graph, initial_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::NodeContent {
                onto: NodeInformation {
                    id: component_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
                to_rebase: NodeInformation {
                    id: component_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
            }],
            conflicts_and_updates.conflicts
        );
        assert!(conflicts_and_updates.updates.is_empty());
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_modify_removed_item_conflict() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_variant_id,
                    Ulid::new(),
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("mark graph seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = base_graph.clone();

        base_graph
            .remove_edge(
                initial_vector_clock_id,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to remove Component A");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("mark graph seen");

        new_graph
            .update_content(
                new_vector_clock_id,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
        new_graph.dot();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(new_vector_clock_id, &base_graph, initial_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        let container = get_root_node_info(&new_graph);

        assert_eq!(
            vec![Conflict::ModifyRemovedItem {
                container,
                modified_item: NodeInformation {
                    id: component_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                }
            }],
            conflicts_and_updates.conflicts
        );
        assert!(conflicts_and_updates.updates.is_empty());
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_modify_removed_item_conflict_same_vector_clocks() {
        let actor_id = Ulid::new();
        let vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraphV1::new(vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    vector_clock_id,
                    schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    vector_clock_id,
                    schema_variant_id,
                    Ulid::new(),
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let component_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    vector_clock_id,
                    component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("Component A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_index,
            )
            .expect("Unable to add root -> component edge");
        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(vector_clock_id)
            .expect("mark graph seen");

        let mut new_graph = base_graph.clone();

        base_graph
            .remove_edge(
                vector_clock_id,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to remove Component A");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(vector_clock_id)
            .expect("mark graph seen");

        new_graph
            .update_content(
                vector_clock_id,
                component_id,
                ContentHash::from("Updated Component A"),
            )
            .expect("Unable to update Component A");

        new_graph.cleanup();
        new_graph.dot();
        new_graph
            .mark_graph_seen(vector_clock_id)
            .expect("unable to mark graph seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(vector_clock_id, &base_graph, vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        let container = get_root_node_info(&new_graph);

        // Even though we have identical vector clocks, this still produces a
        // conflict, since this item has been modified in to_rebase after onto
        // removed it.
        assert_eq!(
            vec![Conflict::ModifyRemovedItem {
                container,
                modified_item: NodeInformation {
                    id: component_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                }
            }],
            conflicts_and_updates.conflicts
        );
        assert!(conflicts_and_updates.updates.is_empty());
    }

    #[test]
    fn detect_conflicts_and_updates_add_unordered_child_to_ordered_container() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let active_graph = &mut base_graph;

        // Create base prop node
        let base_prop_id = {
            let prop_id = active_graph
                .generate_ulid()
                .expect("Unable to generate Ulid");
            let prop_index = active_graph
                .add_ordered_node(
                    initial_vector_clock_id,
                    NodeWeight::new_content(
                        initial_vector_clock_id,
                        prop_id,
                        Ulid::new(),
                        ContentAddress::Prop(ContentHash::new(prop_id.to_string().as_bytes())),
                    )
                    .expect("Unable to create NodeWeight"),
                )
                .expect("Unable to add prop");

            active_graph
                .add_edge(
                    active_graph.root_index,
                    EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                        .expect("Unable to create EdgeWeight"),
                    prop_index,
                )
                .expect("Unable to add sv -> prop edge");

            prop_id
        };

        active_graph.cleanup();

        // Create two prop nodes children of base prop
        let ordered_prop_1_index = {
            let ordered_prop_id = active_graph
                .generate_ulid()
                .expect("Unable to generate Ulid");
            let ordered_prop_index = active_graph
                .add_node(
                    NodeWeight::new_content(
                        initial_vector_clock_id,
                        ordered_prop_id,
                        Ulid::new(),
                        ContentAddress::Prop(ContentHash::new(
                            ordered_prop_id.to_string().as_bytes(),
                        )),
                    )
                    .expect("Unable to create NodeWeight"),
                )
                .expect("Unable to add ordered prop");
            active_graph
                .add_ordered_edge(
                    initial_vector_clock_id,
                    active_graph
                        .get_node_index_by_id(base_prop_id)
                        .expect("Unable to get prop NodeIndex"),
                    EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                        .expect("Unable to create uses edge weight"),
                    ordered_prop_index,
                )
                .expect("Unable to add prop -> ordered_prop_1 edge");

            ordered_prop_index
        };

        active_graph.cleanup();

        let attribute_prototype_id = {
            let node_id = active_graph
                .generate_ulid()
                .expect("Unable to generate Ulid");
            let node_index = active_graph
                .add_node(
                    NodeWeight::new_content(
                        initial_vector_clock_id,
                        node_id,
                        Ulid::new(),
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
                    EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                        .expect("Unable to create EdgeWeight"),
                    node_index,
                )
                .expect("Unable to add root -> prototype edge");

            node_id
        };

        active_graph.cleanup();
        active_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        // Get new graph
        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = base_graph.clone();
        let new_graph = &mut new_graph;

        // Connect Prototype to Prop
        new_graph
            .add_edge(
                new_graph
                    .get_node_index_by_id(base_prop_id)
                    .expect("Unable to get prop NodeIndex"),
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::Prototype(None))
                    .expect("Unable to create EdgeWeight"),
                new_graph
                    .get_node_index_by_id(attribute_prototype_id)
                    .expect("Unable to get prop NodeIndex"),
            )
            .expect("Unable to add sv -> prop edge");
        new_graph.cleanup();
        let base_prop_node_index = new_graph
            .get_node_index_by_id(base_prop_id)
            .expect("Unable to get base prop NodeIndex");

        assert_eq!(
            vec![ordered_prop_1_index,],
            new_graph
                .ordered_children_for_node(base_prop_node_index)
                .expect("Unable to find ordered children for node")
                .expect("Node is not an ordered node")
        );
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");

        // Assert that the new edge to the prototype gets created
        let ConflictsAndUpdates { conflicts, updates } = base_graph
            .detect_conflicts_and_updates(initial_vector_clock_id, new_graph, new_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        assert!(conflicts.is_empty());

        match updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(base_prop_id, source.id.into(),);
                assert_eq!(attribute_prototype_id, destination.id.into(),);
                assert_eq!(&EdgeWeightKind::Prototype(None), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_no_conflicts_no_updates_in_base() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut initial_graph = WorkspaceSnapshotGraphV1::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let schema_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("Schema A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        let schema_variant_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let schema_variant_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    schema_variant_id,
                    Ulid::new(),
                    ContentAddress::SchemaVariant(ContentHash::from("Schema Variant A")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");

        initial_graph
            .add_edge(
                initial_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_index,
            )
            .expect("Unable to add root -> schema edge");
        initial_graph
            .add_edge(
                initial_graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let container_prop_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let container_prop_index = initial_graph
            .add_ordered_node(
                initial_vector_clock_id,
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    container_prop_id,
                    Ulid::new(),
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
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                container_prop_index,
            )
            .expect("Unable to add schema variant -> container prop edge");

        let ordered_prop_1_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_1_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    ordered_prop_1_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_1_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 1");
        initial_graph
            .add_ordered_edge(
                initial_vector_clock_id,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_1_index,
            )
            .expect("Unable to add container prop -> ordered prop 1 edge");

        let ordered_prop_2_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_2_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    ordered_prop_2_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_2_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 2");
        initial_graph
            .add_ordered_edge(
                initial_vector_clock_id,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_2_index,
            )
            .expect("Unable to add container prop -> ordered prop 2 edge");

        let ordered_prop_3_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_3_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    ordered_prop_3_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_3_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 3");
        initial_graph
            .add_ordered_edge(
                initial_vector_clock_id,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_3_index,
            )
            .expect("Unable to add container prop -> ordered prop 3 edge");

        let ordered_prop_4_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_4_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    ordered_prop_4_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_4_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 4");
        initial_graph
            .add_ordered_edge(
                initial_vector_clock_id,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_4_index,
            )
            .expect("Unable to add container prop -> ordered prop 4 edge");

        initial_graph.cleanup();
        initial_graph.dot();

        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = initial_graph.clone();

        let ordered_prop_5_id = new_graph.generate_ulid().expect("Unable to generate Ulid");
        let ordered_prop_5_index = new_graph
            .add_node(
                NodeWeight::new_content(
                    new_vector_clock_id,
                    ordered_prop_5_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        new_graph
            .add_ordered_edge(
                new_vector_clock_id,
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");

        new_graph.cleanup();
        new_graph.dot();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");

        let ConflictsAndUpdates { conflicts, updates } = new_graph
            .detect_conflicts_and_updates(
                new_vector_clock_id,
                &initial_graph,
                initial_vector_clock_id,
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(Vec::<Update>::new(), updates);
    }

    #[test]
    fn simple_ordering_no_conflicts_same_vector_clocks() {
        let vector_clock_id = VectorClockId::new(Ulid::new(), Ulid::new());
        let mut to_rebase_graph = WorkspaceSnapshotGraphV1::new(vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        let ordered_node = "ordered_container";
        let initial_children = vec!["a", "b", "c"];
        let onto_children = vec!["d", "e", "f"];

        let mut node_id_map =
            add_prop_nodes_to_graph(&mut to_rebase_graph, vector_clock_id, &[ordered_node], true);
        node_id_map.extend(add_prop_nodes_to_graph(
            &mut to_rebase_graph,
            vector_clock_id,
            &initial_children,
            false,
        ));
        let ordered_id = *node_id_map.get(ordered_node).expect("should be there");
        let ordered_idx = to_rebase_graph
            .get_node_index_by_id(ordered_id)
            .expect("should have a node index");
        let root_idx = to_rebase_graph.root();
        to_rebase_graph
            .add_edge(
                root_idx,
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                    .expect("failed to make edge weight"),
                ordered_idx,
            )
            .expect("should be able to make an edge");

        for child in &initial_children {
            let ordered_idx = to_rebase_graph
                .get_node_index_by_id(ordered_id)
                .expect("should have a node index");
            let child_id = node_id_map.get(*child).copied().expect("node should exist");
            let child_idx = to_rebase_graph
                .get_node_index_by_id(child_id)
                .expect("should have a node index");
            to_rebase_graph
                .add_ordered_edge(
                    vector_clock_id,
                    ordered_idx,
                    EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                        .expect("failed to make edge weight"),
                    child_idx,
                )
                .expect("should be able to make an edge");
        }

        to_rebase_graph.cleanup();
        to_rebase_graph
            .mark_graph_seen(vector_clock_id)
            .expect("mark twain");

        let mut onto_graph = to_rebase_graph.clone();
        for child in &initial_children {
            let ordered_idx = onto_graph
                .get_node_index_by_id(ordered_id)
                .expect("should have a node index");
            let child_id = node_id_map.get(*child).copied().expect("node should exist");
            let child_idx = onto_graph
                .get_node_index_by_id(child_id)
                .expect("should have a node index");
            onto_graph
                .remove_edge(
                    vector_clock_id,
                    ordered_idx,
                    child_idx,
                    EdgeWeightKindDiscriminants::Use,
                )
                .expect("unable to remove edge");
        }

        node_id_map.extend(add_prop_nodes_to_graph(
            &mut onto_graph,
            vector_clock_id,
            &onto_children,
            false,
        ));

        for child in &onto_children {
            let child_id = node_id_map.get(*child).copied().expect("node should exist");
            let ordered_idx = onto_graph
                .get_node_index_by_id(ordered_id)
                .expect("should have a node index");
            let child_idx = onto_graph
                .get_node_index_by_id(child_id)
                .expect("should have a node index");
            onto_graph
                .add_ordered_edge(
                    vector_clock_id,
                    ordered_idx,
                    EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                        .expect("failed to make edge weight"),
                    child_idx,
                )
                .expect("should be able to make an edge");
        }

        onto_graph.cleanup();
        onto_graph
            .mark_graph_seen(vector_clock_id)
            .expect("call me mark, mr seen is my father");

        let conflicts_and_updates = to_rebase_graph
            .detect_conflicts_and_updates(vector_clock_id, &onto_graph, vector_clock_id)
            .expect("unable to detect conflicts and updates");

        assert!(conflicts_and_updates.conflicts.is_empty());

        to_rebase_graph
            .perform_updates(vector_clock_id, &conflicts_and_updates.updates)
            .expect("unable to perform updates");
        to_rebase_graph.cleanup();

        let ordered_idx = to_rebase_graph
            .get_node_index_by_id(ordered_id)
            .expect("should have a node index");
        let ordering_node = to_rebase_graph
            .ordering_node_for_container(ordered_idx)
            .expect("should not fail")
            .expect("ordering node should exist");

        let expected_order_ids: Vec<Ulid> = onto_children
            .iter()
            .map(|&name| node_id_map.get(name).copied().expect("get id for name"))
            .collect();
        assert_eq!(&expected_order_ids, ordering_node.order());

        let container_children: Vec<Ulid> = to_rebase_graph
            .edges_directed_for_edge_weight_kind(
                ordered_idx,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
            .iter()
            .filter_map(|(_, _, target_idx)| to_rebase_graph.node_index_to_id(*target_idx))
            .collect();

        assert_eq!(expected_order_ids.len(), container_children.len());
        for container_child in &container_children {
            assert!(expected_order_ids.contains(container_child));
        }

        let ordering_node_idx = to_rebase_graph
            .get_node_index_by_id(ordering_node.id())
            .expect("should have an index for ordering node");

        let ordering_node_children: Vec<Ulid> = to_rebase_graph
            .edges_directed(ordering_node_idx, Outgoing)
            .filter_map(|edge_ref| to_rebase_graph.node_index_to_id(edge_ref.target()))
            .collect();

        for child in &ordering_node_children {
            assert!(expected_order_ids.contains(child));
        }
    }

    #[test]
    fn detect_conflicts_and_updates_single_removal_update() {
        let nodes = ["a", "b", "c"];
        let edges = [(None, "a"), (None, "b"), (Some("a"), "c"), (Some("c"), "b")];
        let actor_id = Ulid::new();

        let base_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraphV1::new(base_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Add all nodes from the slice and store their references in a hash map.
        let mut node_id_map = HashMap::new();
        for node in nodes {
            // "props" here are just nodes that are easy to create and render the name on the dot
            // output. there is no domain modeling in this test.
            let node_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
            let prop_node_weight = NodeWeight::new_prop(
                base_vector_clock_id,
                node_id,
                Ulid::new(),
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
                    EdgeWeight::new(base_vector_clock_id, EdgeWeightKind::new_use())
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
            .mark_graph_seen(base_vector_clock_id)
            .expect("could not mark as seen");
        let mut new_graph = base_graph.clone();

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);

        // Remove the first edge involving "c".
        let a_idx = new_graph
            .get_node_index_by_id(a_id)
            .expect("could not get node index by id");
        let c_idx = new_graph
            .get_node_index_by_id(c_id)
            .expect("could not get node index by id");
        new_graph
            .remove_edge(
                new_vector_clock_id,
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
                new_vector_clock_id,
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
            .mark_graph_seen(new_vector_clock_id)
            .expect("could not mark graph seen");
        new_graph.cleanup();

        // base_graph.tiny_dot_to_file(Some("to_rebase"));
        // new_graph.tiny_dot_to_file(Some("onto"));

        let ConflictsAndUpdates { conflicts, updates } = base_graph
            .detect_conflicts_and_updates(base_vector_clock_id, &new_graph, new_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        assert_eq!(
            vec![Update::RemoveEdge {
                source: NodeInformation {
                    id: a_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Prop,
                },
                destination: NodeInformation {
                    id: c_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Prop,
                },
                edge_kind: EdgeWeightKindDiscriminants::Use,
            }],
            updates
        );
        assert!(conflicts.is_empty());
    }

    #[test]
    fn detect_conflicts_and_updates_remove_edge_simple() {
        let actor_id = Ulid::new();
        let to_rebase_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);

        let mut to_rebase_graph = WorkspaceSnapshotGraphV1::new(to_rebase_vector_clock_id)
            .expect("unable to make to_rebase_graph");

        let prototype_node_id = to_rebase_graph.generate_ulid().expect("gen ulid");
        let prototype_node = NodeWeight::new_content(
            to_rebase_vector_clock_id,
            prototype_node_id,
            Ulid::new(),
            ContentAddress::AttributePrototype(ContentHash::from("prototype")),
        )
        .expect("unable to create prototype node weight");

        to_rebase_graph
            .add_node(prototype_node)
            .expect("unable to add node");
        to_rebase_graph
            .add_edge(
                to_rebase_graph.root(),
                EdgeWeight::new(to_rebase_vector_clock_id, EdgeWeightKind::Prototype(None))
                    .expect("make edge weight"),
                to_rebase_graph
                    .get_node_index_by_id(prototype_node_id)
                    .expect("get_node_index_by_id"),
            )
            .expect("unable to add edge");

        // "write" the graph
        to_rebase_graph.cleanup();
        to_rebase_graph
            .mark_graph_seen(to_rebase_vector_clock_id)
            .expect("mark_graph_seen");

        // "fork" a working changeset from the current one
        let onto_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut onto_graph = to_rebase_graph.clone();

        onto_graph
            .remove_edge(
                onto_vector_clock_id,
                onto_graph.root(),
                to_rebase_graph
                    .get_node_index_by_id(prototype_node_id)
                    .expect("get_node_index_by_id"),
                EdgeWeightKindDiscriminants::Prototype,
            )
            .expect("remove_edge");

        onto_graph.cleanup();
        onto_graph
            .mark_graph_seen(onto_vector_clock_id)
            .expect("mark_graph_seen");

        let ConflictsAndUpdates { conflicts, updates } = to_rebase_graph
            .detect_conflicts_and_updates(
                to_rebase_vector_clock_id,
                &onto_graph,
                onto_vector_clock_id,
            )
            .expect("detect_conflicts_and_updates");

        assert!(conflicts.is_empty());
        assert_eq!(1, updates.len());
        assert!(matches!(
            updates[0],
            Update::RemoveEdge {
                edge_kind: EdgeWeightKindDiscriminants::Prototype,
                ..
            }
        ));
    }
}
