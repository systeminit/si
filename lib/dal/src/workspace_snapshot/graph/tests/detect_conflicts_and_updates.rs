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
    use std::collections::HashSet;

    use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
    use crate::workspace_snapshot::edge_weight::{
        EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
    };
    use crate::workspace_snapshot::graph::tests::add_prop_nodes_to_graph;
    use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind::DependentValueRoots;
    use crate::workspace_snapshot::node_weight::NodeWeight;
    use crate::workspace_snapshot::update::Update;
    use crate::workspace_snapshot::{
        conflict::Conflict, graph::ConflictsAndUpdates, vector_clock::HasVectorClocks,
        NodeInformation,
    };
    use crate::NodeWeightDiscriminants;
    use crate::{PropKind, WorkspaceSnapshotGraph};

    #[derive(Debug, Eq, PartialEq)]
    enum UpdateWithEdgeWeightKind {
        NewEdge {
            source: NodeInformation,
            destination: NodeInformation,
            edge_weight_kind: EdgeWeightKind,
        },
        RemoveEdge {
            source: NodeInformation,
            destination: NodeInformation,
            edge_kind: EdgeWeightKindDiscriminants,
        },
        ReplaceSubgraph {
            onto: NodeInformation,
            to_rebase: NodeInformation,
        },
        MergeCategoryNodes {
            to_rebase_category_id: Ulid,
            onto_category_id: Ulid,
        },
    }

    impl From<Update> for UpdateWithEdgeWeightKind {
        fn from(value: Update) -> Self {
            match value {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => Self::NewEdge {
                    source,
                    destination,
                    edge_weight_kind: edge_weight.kind().clone(),
                },
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => Self::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                },
                Update::ReplaceSubgraph { onto, to_rebase } => {
                    Self::ReplaceSubgraph { onto, to_rebase }
                }
                Update::MergeCategoryNodes {
                    to_rebase_category_id,
                    onto_category_id,
                } => Self::MergeCategoryNodes {
                    to_rebase_category_id,
                    onto_category_id,
                },
            }
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_no_updates_in_base() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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

        let mut base_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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

        let new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match conflicts_and_updates.updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(new_graph.root_index, source.index);
                assert_eq!(new_onto_component_index, destination.index);
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_with_purely_new_content_in_new_graph() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);

        let mut base_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(base_graph.root_index, source.index);
                assert_eq!(new_component_index, destination.index);
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_no_conflicts_with_updates_on_both_sides() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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

        let new_onto_component_index = base_graph
            .get_node_index_by_id(new_onto_component_id)
            .expect("Unable to get NodeIndex");
        match conflicts_and_updates.updates.as_slice() {
            [Update::NewEdge {
                source,
                destination,
                edge_weight,
            }] => {
                assert_eq!(new_graph.root_index, source.index);
                assert_eq!(new_onto_component_index, destination.index);
                assert_eq!(&EdgeWeightKind::new_use(), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_content_conflict() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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
                    index: base_graph
                        .get_node_index_by_id(component_id)
                        .expect("Unable to get component NodeIndex"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
                to_rebase: NodeInformation {
                    id: component_id.into(),
                    index: new_graph
                        .get_node_index_by_id(component_id)
                        .expect("Unable to get component NodeIndex"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
            }],
            conflicts_and_updates.conflicts
        );
        assert!(conflicts_and_updates.updates.is_empty());
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_content_conflict_but_no_conflict_same_vector_clocks(
    ) {
        let actor_id = Ulid::new();
        let vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(vector_clock_id)
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
            .expect("mark graph seen");

        base_graph
            .update_content(
                vector_clock_id,
                component_id,
                ContentHash::from("Base Updated Component A"),
            )
            .expect("Unable to update Component A");

        base_graph.cleanup();
        base_graph.dot();
        base_graph
            .mark_graph_seen(vector_clock_id)
            .expect("mark graph seen");

        let conflicts_and_updates = new_graph
            .detect_conflicts_and_updates(vector_clock_id, &base_graph, vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        // With identical vector clocks, we should not hit conflicts
        assert!(conflicts_and_updates.conflicts.is_empty());

        // Instead, we receive the "inverse" of the NodeContent conflict: A
        // ReplaceSubgraph update for the node.
        assert_eq!(
            vec![Update::ReplaceSubgraph {
                onto: NodeInformation {
                    id: component_id.into(),
                    index: base_graph
                        .get_node_index_by_id(component_id)
                        .expect("Unable to get component NodeIndex"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
                to_rebase: NodeInformation {
                    id: component_id.into(),
                    index: new_graph
                        .get_node_index_by_id(component_id)
                        .expect("Unable to get component NodeIndex"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
            }],
            conflicts_and_updates.updates
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_modify_removed_item_conflict() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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

        assert_eq!(
            vec![Conflict::ModifyRemovedItem(NodeInformation {
                id: component_id.into(),
                index: new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                node_weight_kind: NodeWeightDiscriminants::Content,
            })],
            conflicts_and_updates.conflicts
        );
        assert!(conflicts_and_updates.updates.is_empty());
    }

    #[test]
    fn detect_conflicts_and_updates_simple_with_modify_removed_item_conflict_same_vector_clocks() {
        let actor_id = Ulid::new();
        let vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(vector_clock_id)
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

        // Even though we have identical vector clocks, this still produces a
        // conflict, since this item has been modified in to_rebase after onto
        // removed it.
        assert_eq!(
            vec![Conflict::ModifyRemovedItem(NodeInformation {
                id: component_id.into(),
                index: new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                node_weight_kind: NodeWeightDiscriminants::Content,
            })],
            conflicts_and_updates.conflicts
        );
        assert!(conflicts_and_updates.updates.is_empty());
    }

    #[test]
    fn detect_conflicts_and_updates_add_unordered_child_to_ordered_container() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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
                assert_eq!(
                    base_graph
                        .get_node_index_by_id(base_prop_id)
                        .expect("Unable to get prop NodeIndex"),
                    source.index,
                );
                assert_eq!(
                    base_graph
                        .get_node_index_by_id(attribute_prototype_id)
                        .expect("Unable to get prop NodeIndex"),
                    destination.index,
                );
                assert_eq!(&EdgeWeightKind::Prototype(None), edge_weight.kind());
            }
            other => panic!("Unexpected updates: {:?}", other),
        }
    }

    #[test]
    fn detect_conflicts_and_updates_complex() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
            .expect("Unable to create WorkspaceSnapshotGraph");

        // Docker Image Schema
        let docker_image_schema_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let docker_image_schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    docker_image_schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        println!("Add edge from root to {} in onto", docker_image_schema_id);

        // Docker Image Schema Variant
        let docker_image_schema_variant_id =
            base_graph.generate_ulid().expect("Unable to generate Ulid");
        let docker_image_schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    docker_image_schema_variant_id,
                    Ulid::new(),
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
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                docker_image_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!(
            "Add edge from {} to {} in onto",
            docker_image_schema_id, docker_image_schema_variant_id
        );

        // Nginx Docker Image Component
        let nginx_docker_image_component_id =
            base_graph.generate_ulid().expect("Unable to generate Ulid");
        let nginx_docker_image_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    nginx_docker_image_component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                nginx_docker_image_component_index,
            )
            .expect("Unable to add root -> component edge");

        println!(
            "Add edge from root to {} in onto",
            nginx_docker_image_component_id
        );

        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(nginx_docker_image_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!(
            "Add edge from {} to {} in onto",
            nginx_docker_image_component_id, docker_image_schema_variant_id
        );

        // Alpine Component
        let alpine_component_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let alpine_component_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    alpine_component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Component A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                alpine_component_index,
            )
            .expect("Unable to add root -> component edge");

        println!("Add edge from root to {} in onto", alpine_component_id);

        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(alpine_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(docker_image_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!(
            "Add edge from {} to {} in onto",
            alpine_component_id, docker_image_schema_variant_id
        );

        // Butane Schema
        let butane_schema_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let butane_schema_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    butane_schema_id,
                    Ulid::new(),
                    ContentAddress::Schema(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                butane_schema_index,
            )
            .expect("Unable to add root -> schema edge");

        println!("Add edge from root to {} in onto", butane_schema_id);

        // Butane Schema Variant
        let butane_schema_variant_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        let butane_schema_variant_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    butane_schema_variant_id,
                    Ulid::new(),
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
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                butane_schema_variant_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        println!(
            "Add edge from {} to {} in onto",
            butane_schema_id, butane_schema_variant_id
        );

        // Nginx Butane Component
        let nginx_butane_component_id =
            base_graph.generate_ulid().expect("Unable to generate Ulid");
        let nginx_butane_node_index = base_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    nginx_butane_component_id,
                    Ulid::new(),
                    ContentAddress::Component(ContentHash::from("first")),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add Schema Variant A");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                nginx_butane_node_index,
            )
            .expect("Unable to add root -> component edge");

        println!(
            "Add edge from root to {} in onto",
            nginx_butane_component_id
        );

        base_graph
            .add_edge(
                base_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                base_graph
                    .get_node_index_by_id(butane_schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        println!(
            "Add edge from {} to {} in onto",
            nginx_butane_component_id, butane_schema_variant_id
        );

        base_graph.cleanup();
        //base_graph.dot();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        // Create a new change set to cause some problems!
        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = base_graph.clone();

        println!("fork onto into to_rebase");

        // Create a modify removed item conflict.
        base_graph
            .remove_edge(
                initial_vector_clock_id,
                base_graph.root_index,
                base_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to update the component");

        println!(
            "Remove edge from root to {} in onto",
            nginx_butane_component_id
        );

        new_graph
            .update_content(
                new_vector_clock_id,
                nginx_butane_component_id,
                ContentHash::from("second"),
            )
            .expect("Unable to update the component");

        println!(
            "Update content of {} in to_rebase (should produce ModifyRemovedItem conflict)",
            nginx_butane_component_id
        );

        // Create a node content conflict.
        base_graph
            .update_content(
                initial_vector_clock_id,
                docker_image_schema_variant_id,
                ContentHash::from("oopsie"),
            )
            .expect("Unable to update the component");

        println!(
            "Update content of {} in onto",
            docker_image_schema_variant_id
        );

        new_graph
            .update_content(
                new_vector_clock_id,
                docker_image_schema_variant_id,
                ContentHash::from("poopsie"),
            )
            .expect("Unable to update the component");

        println!(
            "Update content of {} in to_rebase (should produce update content conflict)",
            docker_image_schema_variant_id
        );

        // Create a pure update.
        base_graph
            .update_content(
                initial_vector_clock_id,
                docker_image_schema_id,
                ContentHash::from("bg3"),
            )
            .expect("Unable to update the schema");

        println!("Update content of {} in onto", docker_image_schema_id);

        new_graph.cleanup();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");
        base_graph.cleanup();
        base_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        // new_graph.tiny_dot_to_file(Some("to_rebase"));
        // base_graph.tiny_dot_to_file(Some("onto"));

        let ConflictsAndUpdates { conflicts, updates } = new_graph
            .detect_conflicts_and_updates(new_vector_clock_id, &base_graph, initial_vector_clock_id)
            .expect("Unable to detect conflicts and updates");

        // base_graph.dot();
        // new_graph.dot();

        let expected_conflicts = vec![
            Conflict::ModifyRemovedItem(NodeInformation {
                index: new_graph
                    .get_node_index_by_id(nginx_butane_component_id)
                    .expect("Unable to get component NodeIndex"),
                id: nginx_butane_component_id.into(),
                node_weight_kind: NodeWeightDiscriminants::Content,
            }),
            Conflict::NodeContent {
                onto: NodeInformation {
                    index: base_graph
                        .get_node_index_by_id(docker_image_schema_variant_id)
                        .expect("Unable to get component NodeIndex"),
                    id: docker_image_schema_variant_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
                to_rebase: NodeInformation {
                    index: new_graph
                        .get_node_index_by_id(docker_image_schema_variant_id)
                        .expect("Unable to get component NodeIndex"),
                    id: docker_image_schema_variant_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                },
            },
        ];
        let expected_updates = vec![Update::ReplaceSubgraph {
            onto: NodeInformation {
                index: base_graph
                    .get_node_index_by_id(docker_image_schema_id)
                    .expect("Unable to get NodeIndex"),
                id: docker_image_schema_id.into(),
                node_weight_kind: NodeWeightDiscriminants::Content,
            },
            to_rebase: NodeInformation {
                index: new_graph
                    .get_node_index_by_id(docker_image_schema_id)
                    .expect("Unable to get NodeIndex"),
                id: docker_image_schema_id.into(),
                node_weight_kind: NodeWeightDiscriminants::Content,
            },
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
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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
    fn detect_conflicts_and_updates_simple_ordering_no_conflicts_with_updates_in_base() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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

        initial_graph.dot();
        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = initial_graph.clone();

        let ordered_prop_5_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    ordered_prop_5_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        let new_edge_weight = EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
            .expect("Unable to create EdgeWeight");
        let (_, maybe_ordinal_edge_information) = initial_graph
            .add_ordered_edge(
                initial_vector_clock_id,
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
        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");

        new_graph.dot();

        let ConflictsAndUpdates { conflicts, updates } = new_graph
            .detect_conflicts_and_updates(
                new_vector_clock_id,
                &initial_graph,
                initial_vector_clock_id,
            )
            .expect("Unable to detect conflicts and updates");

        let initial_ordering_node_index_for_container = initial_graph
            .ordering_node_index_for_container(
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get container NodeIndex"),
            )
            .expect("Unable to get new ordering NodeIndex")
            .expect("Ordering NodeIndex not found");
        let initial_ordering_node_weight_for_container = initial_graph
            .get_node_weight(initial_ordering_node_index_for_container)
            .expect("Unable to get ordering node weight");
        let new_ordering_node_index_for_container = new_graph
            .ordering_node_index_for_container(
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get container NodeIndex"),
            )
            .expect("Unable to get new ordering NodeIndex")
            .expect("Ordering NodeIndex not found");
        let new_ordering_node_weight_for_container = new_graph
            .get_node_weight(new_ordering_node_index_for_container)
            .expect("Unable to get ordering node weight");
        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(
            vec![
                UpdateWithEdgeWeightKind::NewEdge {
                    source: NodeInformation {
                        index: new_graph
                            .get_node_index_by_id(container_prop_id)
                            .expect("Unable to get NodeIndex"),
                        id: container_prop_id.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    destination: NodeInformation {
                        index: initial_graph
                            .get_node_index_by_id(ordered_prop_5_id)
                            .expect("Unable to get NodeIndex"),
                        id: ordered_prop_5_id.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    edge_weight_kind: new_edge_weight.kind().clone(),
                },
                UpdateWithEdgeWeightKind::ReplaceSubgraph {
                    onto: NodeInformation {
                        index: initial_ordering_node_index_for_container,
                        id: initial_ordering_node_weight_for_container.id().into(),
                        node_weight_kind: NodeWeightDiscriminants::Ordering,
                    },
                    to_rebase: NodeInformation {
                        index: new_ordering_node_index_for_container,
                        id: new_ordering_node_weight_for_container.id().into(),
                        node_weight_kind: NodeWeightDiscriminants::Ordering,
                    },
                },
                UpdateWithEdgeWeightKind::NewEdge {
                    source: NodeInformation {
                        index: new_graph
                            .get_node_index_by_id(source_node_id_for_ordinal_edge)
                            .expect("could not get node index by id"),
                        id: source_node_id_for_ordinal_edge.into(),
                        node_weight_kind: NodeWeightDiscriminants::Ordering,
                    },
                    destination: NodeInformation {
                        index: initial_graph
                            .get_node_index_by_id(destination_node_id_for_ordinal_edge)
                            .expect("could not get node index by id"),
                        id: destination_node_id_for_ordinal_edge.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    edge_weight_kind: ordinal_edge_weight.kind().clone(),
                }
            ],
            updates
                .into_iter()
                .map(Into::<UpdateWithEdgeWeightKind>::into)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn detect_conflicts_and_updates_simple_ordering_with_conflicting_ordering_updates() {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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

        initial_graph.dot();
        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_graph = initial_graph.clone();

        let new_order = vec![
            ordered_prop_2_id,
            ordered_prop_1_id,
            ordered_prop_4_id,
            ordered_prop_3_id,
        ];
        new_graph
            .update_order(new_vector_clock_id, container_prop_id, new_order)
            .expect("Unable to update order of container prop's children");

        let ordered_prop_5_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    ordered_prop_5_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");
        let new_edge_weight = EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
            .expect("Unable to create EdgeWeight");
        let (_, maybe_ordinal_edge_information) = initial_graph
            .add_ordered_edge(
                initial_vector_clock_id,
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

        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable to mark graph seen");

        new_graph.cleanup();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");
        new_graph.dot();

        let ConflictsAndUpdates { conflicts, updates } = new_graph
            .detect_conflicts_and_updates(
                new_vector_clock_id,
                &initial_graph,
                initial_vector_clock_id,
            )
            .expect("Unable to detect conflicts and updates");

        let initial_container_ordering_node_index = initial_graph
            .ordering_node_index_for_container(
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get container node index"),
            )
            .expect("Unable to get ordering node index")
            .expect("No ordering node");
        let new_container_ordering_node_index = new_graph
            .ordering_node_index_for_container(
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get container node index"),
            )
            .expect("Unable to get ordering node index")
            .expect("No ordering node");

        assert_eq!(
            vec![Conflict::ChildOrder {
                onto: NodeInformation {
                    index: initial_container_ordering_node_index,
                    id: initial_graph
                        .get_node_weight(initial_container_ordering_node_index)
                        .expect("Unable to get ordering node")
                        .id()
                        .into(),
                    node_weight_kind: NodeWeightDiscriminants::Ordering,
                },
                to_rebase: NodeInformation {
                    index: new_container_ordering_node_index,
                    id: new_graph
                        .get_node_weight(new_container_ordering_node_index)
                        .expect("Unable to get new ordering node")
                        .id()
                        .into(),
                    node_weight_kind: NodeWeightDiscriminants::Ordering,
                },
            }],
            conflicts
        );

        assert_eq!(
            vec![
                UpdateWithEdgeWeightKind::NewEdge {
                    source: NodeInformation {
                        index: new_graph
                            .get_node_index_by_id(container_prop_id)
                            .expect("Unable to get new prop index"),
                        id: container_prop_id.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    destination: NodeInformation {
                        index: initial_graph
                            .get_node_index_by_id(ordered_prop_5_id)
                            .expect("Unable to get ordered prop 5 index"),
                        id: ordered_prop_5_id.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    edge_weight_kind: new_edge_weight.kind().clone(),
                },
                UpdateWithEdgeWeightKind::NewEdge {
                    source: NodeInformation {
                        index: new_graph
                            .get_node_index_by_id(source_node_id_for_ordinal_edge)
                            .expect("could not get node index by id"),
                        id: source_node_id_for_ordinal_edge.into(),
                        node_weight_kind: NodeWeightDiscriminants::Ordering,
                    },
                    destination: NodeInformation {
                        index: initial_graph
                            .get_node_index_by_id(destination_node_id_for_ordinal_edge)
                            .expect("could not get node index by id"),
                        id: destination_node_id_for_ordinal_edge.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    edge_weight_kind: ordinal_edge_weight.kind().to_owned(),
                }
            ],
            updates
                .into_iter()
                .map(Into::<UpdateWithEdgeWeightKind>::into)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn simple_ordering_no_conflicts_same_vector_clocks() {
        let vector_clock_id = VectorClockId::new(Ulid::new(), Ulid::new());
        let mut to_rebase_graph = WorkspaceSnapshotGraph::new(vector_clock_id)
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
            .perform_updates(vector_clock_id, &onto_graph, &conflicts_and_updates.updates)
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
    fn detect_conflicts_and_updates_simple_ordering_with_no_conflicts_add_in_onto_remove_in_to_rebase(
    ) {
        let actor_id = Ulid::new();
        let initial_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut initial_graph = WorkspaceSnapshotGraph::new(initial_vector_clock_id)
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

        println!("added ordered edge from {container_prop_id} to {ordered_prop_1_id} in onto");

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

        println!("added ordered edge from {container_prop_id} to {ordered_prop_2_id} in onto");

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

        println!("added ordered edge from {container_prop_id} to {ordered_prop_3_id} in onto");

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

        println!("added ordered edge from {container_prop_id} to {ordered_prop_4_id} in onto");

        initial_graph.cleanup();
        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("Unable to update recently seen information");
        // initial_graph.dot();

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);

        let mut new_graph = initial_graph.clone();

        new_graph
            .remove_edge(
                new_vector_clock_id,
                new_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get container NodeIndex"),
                ordered_prop_2_index,
                EdgeWeightKindDiscriminants::Use,
            )
            .expect("Unable to remove container prop -> prop 2 edge");

        println!("removed edge from {container_prop_id} to {ordered_prop_2_id} in to_rebase");

        let ordered_prop_5_id = initial_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        let ordered_prop_5_index = initial_graph
            .add_node(
                NodeWeight::new_content(
                    initial_vector_clock_id,
                    ordered_prop_5_id,
                    Ulid::new(),
                    ContentAddress::Prop(ContentHash::new(
                        ordered_prop_5_id.to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ordered prop 5");

        let new_edge_weight = EdgeWeight::new(initial_vector_clock_id, EdgeWeightKind::new_use())
            .expect("Unable to create EdgeWeight");
        let (_, maybe_ordinal_edge_information) = initial_graph
            .add_ordered_edge(
                initial_vector_clock_id,
                initial_graph
                    .get_node_index_by_id(container_prop_id)
                    .expect("Unable to get NodeIndex"),
                new_edge_weight.clone(),
                ordered_prop_5_index,
            )
            .expect("Unable to add container prop -> ordered prop 5 edge");
        println!("added ordered edge from {container_prop_id} to {ordered_prop_5_id} in onto");
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
        //initial_graph.dot();
        initial_graph
            .mark_graph_seen(initial_vector_clock_id)
            .expect("unable mark graph seen");

        new_graph.cleanup();
        //new_graph.dot();
        new_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("unable to mark graph seen");

        // initial_graph.tiny_dot_to_file(Some("onto"));
        // new_graph.tiny_dot_to_file(Some("to_rebase"));

        let ConflictsAndUpdates { conflicts, updates } = new_graph
            .detect_conflicts_and_updates(
                new_vector_clock_id,
                &initial_graph,
                initial_vector_clock_id,
            )
            .expect("Unable to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);
        assert_eq!(
            vec![
                UpdateWithEdgeWeightKind::NewEdge {
                    source: NodeInformation {
                        index: new_graph
                            .get_node_index_by_id(container_prop_id)
                            .expect("Unable to get new_graph container NodeIndex"),
                        id: container_prop_id.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    destination: NodeInformation {
                        index: initial_graph
                            .get_node_index_by_id(ordered_prop_5_id)
                            .expect("Unable to get ordered prop 5 NodeIndex"),
                        id: ordered_prop_5_id.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    edge_weight_kind: new_edge_weight.kind().clone(),
                },
                UpdateWithEdgeWeightKind::NewEdge {
                    source: NodeInformation {
                        index: new_graph
                            .get_node_index_by_id(source_node_id_for_ordinal_edge)
                            .expect("could not get node index by id"),
                        id: source_node_id_for_ordinal_edge.into(),
                        node_weight_kind: NodeWeightDiscriminants::Ordering,
                    },
                    destination: NodeInformation {
                        index: initial_graph
                            .get_node_index_by_id(destination_node_id_for_ordinal_edge)
                            .expect("could not get node index by id"),
                        id: destination_node_id_for_ordinal_edge.into(),
                        node_weight_kind: NodeWeightDiscriminants::Content,
                    },
                    edge_weight_kind: ordinal_edge_weight.kind().clone(),
                }
            ],
            updates
                .into_iter()
                .map(Into::<UpdateWithEdgeWeightKind>::into)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn detect_conflicts_and_updates_single_removal_update() {
        let nodes = ["a", "b", "c"];
        let edges = [(None, "a"), (None, "b"), (Some("a"), "c"), (Some("c"), "b")];
        let actor_id = Ulid::new();

        let base_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph = WorkspaceSnapshotGraph::new(base_vector_clock_id)
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
                    index: a_idx,
                    id: a_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Prop,
                },
                destination: NodeInformation {
                    index: c_idx,
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
    fn detect_exclusive_edge_conflict() {
        let actor_id = Ulid::new();
        let base_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph =
            WorkspaceSnapshotGraph::new(base_vector_clock_id).expect("Unable to make base graph");

        let av_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        base_graph
            .add_node(
                NodeWeight::new_attribute_value(
                    base_vector_clock_id,
                    av_id,
                    Ulid::new(),
                    None,
                    None,
                )
                .expect("Unable to create AttributeValue"),
            )
            .expect("Unable to add AttributeValue");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(base_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create edge weight"),
                base_graph
                    .get_node_index_by_id(av_id)
                    .expect("Unable to get node index for AttributeValue"),
            )
            .expect("Unable to add root -> AV Use edge");
        base_graph.cleanup();
        base_graph
            .mark_graph_seen(base_vector_clock_id)
            .expect("Unable to mark base graph as seen");

        let new_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut new_changes_graph = base_graph.clone();

        let prototype_a_id = new_changes_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        new_changes_graph
            .add_node(
                NodeWeight::new_content(
                    new_vector_clock_id,
                    prototype_a_id,
                    Ulid::new(),
                    ContentAddress::AttributePrototype(ContentHash::new(
                        "Prototype A".to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create content node weight"),
            )
            .expect("Unable to add prototype a to graph");
        new_changes_graph
            .add_edge(
                new_changes_graph
                    .get_node_index_by_id(av_id)
                    .expect("Unable to get AV node index"),
                EdgeWeight::new(new_vector_clock_id, EdgeWeightKind::Prototype(None))
                    .expect("Unable to create edge weight"),
                new_changes_graph
                    .get_node_index_by_id(prototype_a_id)
                    .expect("Unable to get node index for AttributePrototype"),
            )
            .expect("Unable to add AV -> AttributePrototype Prototype edge");
        new_changes_graph.cleanup();
        new_changes_graph
            .mark_graph_seen(new_vector_clock_id)
            .expect("Unable to mark new changes graph as seen");

        let ConflictsAndUpdates {
            conflicts,
            updates: _,
        } = base_graph
            .detect_conflicts_and_updates(
                base_vector_clock_id,
                &new_changes_graph,
                new_vector_clock_id,
            )
            .expect("able to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);

        let conflicting_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut conflicting_graph = new_changes_graph.clone();

        let prototype_b_id = conflicting_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        conflicting_graph
            .add_node(
                NodeWeight::new_content(
                    conflicting_vector_clock_id,
                    prototype_b_id,
                    Ulid::new(),
                    ContentAddress::AttributePrototype(ContentHash::new(
                        "Prototype B".to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create content node weight"),
            )
            .expect("Unable to add prototype b to graph");
        conflicting_graph
            .add_edge(
                conflicting_graph
                    .get_node_index_by_id(av_id)
                    .expect("Unable to get AV node index"),
                EdgeWeight::new(conflicting_vector_clock_id, EdgeWeightKind::Prototype(None))
                    .expect("Unable to create edge weight"),
                conflicting_graph
                    .get_node_index_by_id(prototype_b_id)
                    .expect("Unable to get node index for AttributePrototype"),
            )
            .expect("Unable to add AV -> AttributePrototype Prototype edge");

        conflicting_graph.cleanup();
        conflicting_graph
            .mark_graph_seen(conflicting_vector_clock_id)
            .expect("Unable to mark conflicting graph as seen");

        let ConflictsAndUpdates {
            conflicts,
            updates: _,
        } = new_changes_graph
            .detect_conflicts_and_updates(
                new_vector_clock_id,
                &conflicting_graph,
                conflicting_vector_clock_id,
            )
            .expect("able to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::ExclusiveEdgeMismatch {
                source: NodeInformation {
                    index: new_changes_graph
                        .get_node_index_by_id(av_id)
                        .expect("Unable to get AV node index"),
                    node_weight_kind: NodeWeightDiscriminants::AttributeValue,
                    id: av_id.into(),
                },
                destination: NodeInformation {
                    index: conflicting_graph
                        .get_node_index_by_id(prototype_b_id)
                        .expect("Unable to get Prototype B node index"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                    id: prototype_b_id.into(),
                },
                edge_kind: EdgeWeightKindDiscriminants::Prototype,
            }],
            conflicts,
        );

        let ConflictsAndUpdates {
            conflicts,
            updates: _,
        } = base_graph
            .detect_conflicts_and_updates(
                base_vector_clock_id,
                &conflicting_graph,
                conflicting_vector_clock_id,
            )
            .expect("able to detect conflicts and updates");

        let expected_conflicts: HashSet<Conflict> = [
            Conflict::ExclusiveEdgeMismatch {
                source: NodeInformation {
                    index: base_graph
                        .get_node_index_by_id(av_id)
                        .expect("Unable to get AV node index"),
                    node_weight_kind: NodeWeightDiscriminants::AttributeValue,
                    id: av_id.into(),
                },
                destination: NodeInformation {
                    index: conflicting_graph
                        .get_node_index_by_id(prototype_a_id)
                        .expect("Unable to get Prototype A node index"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                    id: prototype_a_id.into(),
                },
                edge_kind: EdgeWeightKindDiscriminants::Prototype,
            },
            Conflict::ExclusiveEdgeMismatch {
                source: NodeInformation {
                    index: base_graph
                        .get_node_index_by_id(av_id)
                        .expect("Unable to get AV node index"),
                    node_weight_kind: NodeWeightDiscriminants::AttributeValue,
                    id: av_id.into(),
                },
                destination: NodeInformation {
                    index: conflicting_graph
                        .get_node_index_by_id(prototype_b_id)
                        .expect("Unable to get Prototype B node index"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                    id: prototype_b_id.into(),
                },
                edge_kind: EdgeWeightKindDiscriminants::Prototype,
            },
        ]
        .iter()
        .copied()
        .collect();
        let actual_conflicts: HashSet<Conflict> = conflicts.iter().copied().collect();

        assert_eq!(expected_conflicts, actual_conflicts);
    }

    #[test]
    fn detect_exclusive_edge_conflict_same_vector_clocks() {
        let actor_id = Ulid::new();
        let vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut base_graph =
            WorkspaceSnapshotGraph::new(vector_clock_id).expect("Unable to make base graph");

        let av_id = base_graph.generate_ulid().expect("Unable to generate Ulid");
        base_graph
            .add_node(
                NodeWeight::new_attribute_value(vector_clock_id, av_id, Ulid::new(), None, None)
                    .expect("Unable to create AttributeValue"),
            )
            .expect("Unable to add AttributeValue");
        base_graph
            .add_edge(
                base_graph.root_index,
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                    .expect("Unable to create edge weight"),
                base_graph
                    .get_node_index_by_id(av_id)
                    .expect("Unable to get node index for AttributeValue"),
            )
            .expect("Unable to add root -> AV Use edge");
        base_graph.cleanup();
        base_graph
            .mark_graph_seen(vector_clock_id)
            .expect("Unable to mark base graph as seen");

        let mut new_changes_graph = base_graph.clone();

        let prototype_a_id = new_changes_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        new_changes_graph
            .add_node(
                NodeWeight::new_content(
                    vector_clock_id,
                    prototype_a_id,
                    Ulid::new(),
                    ContentAddress::AttributePrototype(ContentHash::new(
                        "Prototype A".to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create content node weight"),
            )
            .expect("Unable to add prototype a to graph");
        new_changes_graph
            .add_edge(
                new_changes_graph
                    .get_node_index_by_id(av_id)
                    .expect("Unable to get AV node index"),
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::Prototype(None))
                    .expect("Unable to create edge weight"),
                new_changes_graph
                    .get_node_index_by_id(prototype_a_id)
                    .expect("Unable to get node index for AttributePrototype"),
            )
            .expect("Unable to add AV -> AttributePrototype Prototype edge");
        new_changes_graph.cleanup();
        new_changes_graph
            .mark_graph_seen(vector_clock_id)
            .expect("Unable to mark new changes graph as seen");

        let ConflictsAndUpdates {
            conflicts,
            updates: _,
        } = base_graph
            .detect_conflicts_and_updates(vector_clock_id, &new_changes_graph, vector_clock_id)
            .expect("able to detect conflicts and updates");

        assert_eq!(Vec::<Conflict>::new(), conflicts);

        let mut conflicting_graph = new_changes_graph.clone();

        let prototype_b_id = conflicting_graph
            .generate_ulid()
            .expect("Unable to generate Ulid");
        conflicting_graph
            .add_node(
                NodeWeight::new_content(
                    vector_clock_id,
                    prototype_b_id,
                    Ulid::new(),
                    ContentAddress::AttributePrototype(ContentHash::new(
                        "Prototype B".to_string().as_bytes(),
                    )),
                )
                .expect("Unable to create content node weight"),
            )
            .expect("Unable to add prototype b to graph");
        conflicting_graph
            .add_edge(
                conflicting_graph
                    .get_node_index_by_id(av_id)
                    .expect("Unable to get AV node index"),
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::Prototype(None))
                    .expect("Unable to create edge weight"),
                conflicting_graph
                    .get_node_index_by_id(prototype_b_id)
                    .expect("Unable to get node index for AttributePrototype"),
            )
            .expect("Unable to add AV -> AttributePrototype Prototype edge");

        conflicting_graph.cleanup();
        conflicting_graph
            .mark_graph_seen(vector_clock_id)
            .expect("Unable to mark conflicting graph as seen");

        let ConflictsAndUpdates {
            conflicts,
            updates: _,
        } = new_changes_graph
            .detect_conflicts_and_updates(vector_clock_id, &conflicting_graph, vector_clock_id)
            .expect("able to detect conflicts and updates");

        assert_eq!(
            vec![Conflict::ExclusiveEdgeMismatch {
                source: NodeInformation {
                    index: new_changes_graph
                        .get_node_index_by_id(av_id)
                        .expect("Unable to get AV node index"),
                    node_weight_kind: NodeWeightDiscriminants::AttributeValue,
                    id: av_id.into(),
                },
                destination: NodeInformation {
                    index: conflicting_graph
                        .get_node_index_by_id(prototype_b_id)
                        .expect("Unable to get Prototype B node index"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                    id: prototype_b_id.into(),
                },
                edge_kind: EdgeWeightKindDiscriminants::Prototype,
            }],
            conflicts,
        );

        let ConflictsAndUpdates {
            conflicts,
            updates: _,
        } = base_graph
            .detect_conflicts_and_updates(vector_clock_id, &conflicting_graph, vector_clock_id)
            .expect("able to detect conflicts and updates");

        let expected_conflicts: HashSet<Conflict> = [
            Conflict::ExclusiveEdgeMismatch {
                source: NodeInformation {
                    index: base_graph
                        .get_node_index_by_id(av_id)
                        .expect("Unable to get AV node index"),
                    node_weight_kind: NodeWeightDiscriminants::AttributeValue,
                    id: av_id.into(),
                },
                destination: NodeInformation {
                    index: conflicting_graph
                        .get_node_index_by_id(prototype_a_id)
                        .expect("Unable to get Prototype A node index"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                    id: prototype_a_id.into(),
                },
                edge_kind: EdgeWeightKindDiscriminants::Prototype,
            },
            Conflict::ExclusiveEdgeMismatch {
                source: NodeInformation {
                    index: base_graph
                        .get_node_index_by_id(av_id)
                        .expect("Unable to get AV node index"),
                    node_weight_kind: NodeWeightDiscriminants::AttributeValue,
                    id: av_id.into(),
                },
                destination: NodeInformation {
                    index: conflicting_graph
                        .get_node_index_by_id(prototype_b_id)
                        .expect("Unable to get Prototype B node index"),
                    node_weight_kind: NodeWeightDiscriminants::Content,
                    id: prototype_b_id.into(),
                },
                edge_kind: EdgeWeightKindDiscriminants::Prototype,
            },
        ]
        .iter()
        .copied()
        .collect();
        let actual_conflicts: HashSet<Conflict> = conflicts.iter().copied().collect();

        assert_eq!(expected_conflicts, actual_conflicts);
    }

    #[test]
    fn detect_conflicts_and_updates_remove_edge_simple() {
        let actor_id = Ulid::new();
        let to_rebase_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);

        let mut to_rebase_graph = WorkspaceSnapshotGraph::new(to_rebase_vector_clock_id)
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

    #[test]
    fn detect_conflicts_and_updates_remove_modified_item_conflict() {
        let actor_id = Ulid::new();
        let to_rebase_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut to_rebase_graph = WorkspaceSnapshotGraph::new(to_rebase_vector_clock_id)
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

        // After the fork, remove the edge in to_rebase, but modify the edge in onto
        to_rebase_graph
            .remove_edge(
                onto_vector_clock_id,
                onto_graph.root(),
                to_rebase_graph
                    .get_node_index_by_id(prototype_node_id)
                    .expect("get_node_index_by_id"),
                EdgeWeightKindDiscriminants::Prototype,
            )
            .expect("remove_edge");
        to_rebase_graph.cleanup();
        to_rebase_graph
            .mark_graph_seen(to_rebase_vector_clock_id)
            .expect("mark_graph_seen");

        let onto_content_node_idx = onto_graph
            .get_node_index_by_id(prototype_node_id)
            .expect("get_node_index_by_id");

        let mut content_node = onto_graph
            .get_node_weight(onto_content_node_idx)
            .expect("get_node_weight")
            .get_content_node_weight_of_kind(ContentAddressDiscriminants::AttributePrototype)
            .expect("get_content_node_weight_of_kind");

        // Modifying this node in onto, after it has been removed in to_rebase,
        // will produce a RemoveModifiedItem conflict
        content_node
            .new_content_hash(ContentHash::from("prototype_change"))
            .expect("update_content_hash");
        content_node.increment_vector_clocks(onto_vector_clock_id);
        onto_graph
            .add_node(NodeWeight::Content(content_node))
            .expect("add_node");
        onto_graph
            .replace_references(onto_content_node_idx)
            .expect("replace_references");
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

        // Since the node in question is removed in to_rebase, there will be no
        // ReplaceSubgraph update
        assert!(updates.is_empty());
        assert_eq!(1, conflicts.len());

        let container = NodeInformation {
            index: to_rebase_graph.root_index,
            id: to_rebase_graph
                .get_node_weight(to_rebase_graph.root())
                .expect("Unable to get root node")
                .id()
                .into(),
            node_weight_kind: NodeWeightDiscriminants::Content,
        };
        let removed_index = onto_graph
            .get_node_index_by_id(prototype_node_id)
            .expect("get_node_index_by_id");
        let removed_item = NodeInformation {
            index: removed_index,
            id: onto_graph
                .get_node_weight(removed_index)
                .expect("Unable to get removed item node weight")
                .id()
                .into(),
            node_weight_kind: NodeWeightDiscriminants::Content,
        };
        assert_eq!(
            conflicts[0],
            Conflict::RemoveModifiedItem {
                container,
                removed_item
            }
        );
    }

    #[test]
    fn detect_conflicts_and_updates_remove_modified_item_conflict_same_vector_clocks() {
        let actor_id = Ulid::new();
        let vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut to_rebase_graph =
            WorkspaceSnapshotGraph::new(vector_clock_id).expect("unable to make to_rebase_graph");

        let prototype_node_id = to_rebase_graph.generate_ulid().expect("gen ulid");
        let prototype_node = NodeWeight::new_content(
            vector_clock_id,
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
                EdgeWeight::new(vector_clock_id, EdgeWeightKind::Prototype(None))
                    .expect("make edge weight"),
                to_rebase_graph
                    .get_node_index_by_id(prototype_node_id)
                    .expect("get_node_index_by_id"),
            )
            .expect("unable to add edge");

        // "write" the graph
        to_rebase_graph.cleanup();
        to_rebase_graph
            .mark_graph_seen(vector_clock_id)
            .expect("mark_graph_seen");

        // "fork" a working changeset from the current one
        let mut onto_graph = to_rebase_graph.clone();

        // After the fork, remove the edge in to_rebase, but modify the edge in onto
        to_rebase_graph
            .remove_edge(
                vector_clock_id,
                onto_graph.root(),
                to_rebase_graph
                    .get_node_index_by_id(prototype_node_id)
                    .expect("get_node_index_by_id"),
                EdgeWeightKindDiscriminants::Prototype,
            )
            .expect("remove_edge");
        to_rebase_graph.cleanup();
        to_rebase_graph
            .mark_graph_seen(vector_clock_id)
            .expect("mark_graph_seen");

        let onto_content_node_idx = onto_graph
            .get_node_index_by_id(prototype_node_id)
            .expect("get_node_index_by_id");

        let mut content_node = onto_graph
            .get_node_weight(onto_content_node_idx)
            .expect("get_node_weight")
            .get_content_node_weight_of_kind(ContentAddressDiscriminants::AttributePrototype)
            .expect("get_content_node_weight_of_kind");

        // Modifying this node in onto, after it has been removed in to_rebase
        // will produce a RemoveModifiedItem conflict, even though the vector
        // clocks are the same, since the same conditions still hold.
        content_node
            .new_content_hash(ContentHash::from("prototype_change"))
            .expect("update_content_hash");
        content_node.increment_vector_clocks(vector_clock_id);
        onto_graph
            .add_node(NodeWeight::Content(content_node))
            .expect("add_node");
        onto_graph
            .replace_references(onto_content_node_idx)
            .expect("replace_references");
        onto_graph.cleanup();
        onto_graph
            .mark_graph_seen(vector_clock_id)
            .expect("mark_graph_seen");

        let ConflictsAndUpdates { conflicts, updates } = to_rebase_graph
            .detect_conflicts_and_updates(vector_clock_id, &onto_graph, vector_clock_id)
            .expect("detect_conflicts_and_updates");

        assert!(updates.is_empty());
        assert_eq!(1, conflicts.len());

        let container = NodeInformation {
            index: to_rebase_graph.root_index,
            id: to_rebase_graph
                .get_node_weight(to_rebase_graph.root())
                .expect("Unable to get root node")
                .id()
                .into(),
            node_weight_kind: NodeWeightDiscriminants::Content,
        };
        let removed_index = onto_graph
            .get_node_index_by_id(prototype_node_id)
            .expect("get_node_index_by_id");
        let removed_item = NodeInformation {
            index: removed_index,
            id: onto_graph
                .get_node_weight(removed_index)
                .expect("Unable to get removed item node weight")
                .id()
                .into(),
            node_weight_kind: NodeWeightDiscriminants::Content,
        };
        assert_eq!(
            Conflict::RemoveModifiedItem {
                container,
                removed_item,
            },
            conflicts[0],
        );

        // Reversing the detection order should produce the ModifyRemovedItem conflict
        let ConflictsAndUpdates { conflicts, updates } = onto_graph
            .detect_conflicts_and_updates(vector_clock_id, &to_rebase_graph, vector_clock_id)
            .expect("detect_conflicts_and_updates");
        assert!(updates.is_empty());
        assert_eq!(Conflict::ModifyRemovedItem(removed_item), conflicts[0]);
    }

    #[test]
    fn test_merge_dependent_value_roots() {
        let actor_id = Ulid::new();
        let to_rebase_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut to_rebase_graph = WorkspaceSnapshotGraph::new(to_rebase_vector_clock_id)
            .expect("unable to make to_rebase_graph");

        to_rebase_graph
            .mark_graph_seen(to_rebase_vector_clock_id)
            .expect("unable to mark graph seen");

        let onto_vector_clock_id = VectorClockId::new(Ulid::new(), actor_id);
        let mut onto_graph = to_rebase_graph.clone();

        let cat_node_idx = to_rebase_graph
            .add_category_node(
                to_rebase_vector_clock_id,
                Ulid::new(),
                Ulid::new(),
                DependentValueRoots,
            )
            .expect("able to add dvu root cat node");
        let to_rebase_cat_node_orig_weight = to_rebase_graph
            .get_node_weight(cat_node_idx)
            .expect("unable to get node weight")
            .to_owned();
        to_rebase_graph
            .add_edge(
                to_rebase_graph.root_index,
                EdgeWeight::new(to_rebase_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("unable to make edge weigh"),
                cat_node_idx,
            )
            .expect("unable add edge ");

        let cat_node_idx = onto_graph
            .add_category_node(
                onto_vector_clock_id,
                Ulid::new(),
                Ulid::new(),
                DependentValueRoots,
            )
            .expect("unable to add dvu root cat node");
        onto_graph
            .add_edge(
                onto_graph.root_index,
                EdgeWeight::new(onto_vector_clock_id, EdgeWeightKind::new_use())
                    .expect("unable to make edge weigh"),
                cat_node_idx,
            )
            .expect("unable add edge ");

        let shared_value_id = to_rebase_graph.generate_ulid().expect("unable to gen ulid");

        let unique_to_rebase_value_id = to_rebase_graph
            .generate_ulid()
            .expect("unable to generate ulid");

        let unique_to_onto_value_id = onto_graph.generate_ulid().expect("unable to generate ulid");

        let to_rebase_value_ids = [unique_to_rebase_value_id, shared_value_id];
        let onto_value_ids = [unique_to_onto_value_id, shared_value_id];

        for (graph, vector_clock_id, values) in [
            (
                &mut to_rebase_graph,
                to_rebase_vector_clock_id,
                &to_rebase_value_ids,
            ),
            (&mut onto_graph, onto_vector_clock_id, &onto_value_ids),
        ] {
            let (cat_id, _) = graph
                .get_category_node(None, DependentValueRoots)
                .expect("unable to get cat node")
                .expect("cat node for dvu roots not there");

            for value_id in values {
                let node_weight = NodeWeight::new_dependent_value_root(
                    vector_clock_id,
                    Ulid::new(),
                    Ulid::new(),
                    *value_id,
                )
                .expect("unable to make root node weight");
                let dvu_root_idx = graph.add_node(node_weight).expect("unable to add node");
                graph
                    .add_edge(
                        graph
                            .get_node_index_by_id(cat_id)
                            .expect("unable to get node index for category"),
                        EdgeWeight::new(vector_clock_id, EdgeWeightKind::new_use())
                            .expect("unable to make edge weight"),
                        dvu_root_idx,
                    )
                    .expect("unable to add edge");
            }
        }

        to_rebase_graph.cleanup();
        onto_graph.cleanup();
        to_rebase_graph
            .mark_graph_seen(to_rebase_vector_clock_id)
            .expect("unable to mark graph seen");
        onto_graph
            .mark_graph_seen(onto_vector_clock_id)
            .expect("unable to mark graph seen");

        let ConflictsAndUpdates { conflicts, updates } = to_rebase_graph
            .detect_conflicts_and_updates(
                to_rebase_vector_clock_id,
                &onto_graph,
                onto_vector_clock_id,
            )
            .expect("able to detect conflicts and updates");

        assert!(conflicts.is_empty());

        to_rebase_graph
            .perform_updates(to_rebase_vector_clock_id, &onto_graph, &updates)
            .expect("unable to perform updates");

        to_rebase_graph.cleanup();
        to_rebase_graph
            .mark_graph_seen(to_rebase_vector_clock_id)
            .expect("unable to mark graph seen");

        let neighbors_of_root: Vec<NodeIndex> = to_rebase_graph
            .edges_directed(to_rebase_graph.root_index, Outgoing)
            .map(|edge_ref| edge_ref.target())
            .collect();

        assert_eq!(1, neighbors_of_root.len());
        let cat_node_idx = neighbors_of_root[0];
        let cat_node_weight = to_rebase_graph
            .get_node_weight(cat_node_idx)
            .expect("unable to get cat node weight")
            .to_owned();

        assert_eq!(to_rebase_cat_node_orig_weight.id(), cat_node_weight.id());
        let neighbors_of_cat: Vec<NodeIndex> = to_rebase_graph
            .edges_directed(cat_node_idx, Outgoing)
            .map(|edge_ref| edge_ref.target())
            .collect();

        assert_eq!(4, neighbors_of_cat.len());
        let mut expected_id_set: HashSet<Ulid> = HashSet::new();
        expected_id_set.extend(&to_rebase_value_ids);
        expected_id_set.extend(&onto_value_ids);

        let mut found_id_set = HashSet::new();
        for neighbor_idx in neighbors_of_cat {
            let node_weight = to_rebase_graph
                .get_node_weight(neighbor_idx)
                .expect("unable to get node weight")
                .get_dependent_value_root_node_weight()
                .expect("unable to get dvu root");

            found_id_set.insert(node_weight.value_id());
        }

        assert_eq!(expected_id_set, found_id_set);
    }
}
