#[allow(clippy::panic)]
#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
mod test {
    use si_events::{ulid::Ulid, ContentHash};

    use crate::{
        action::prototype::ActionKind,
        workspace_snapshot::{
            content_address::ContentAddress,
            graph::{detector::Update, WorkspaceSnapshotGraphResult},
            node_weight::{
                traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms},
                NodeWeight,
            },
        },
        EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants, NodeWeightDiscriminants,
        WorkspaceSnapshotGraphVCurrent,
    };

    #[test]
    fn correct_exclusive_outgoing_edges() -> WorkspaceSnapshotGraphResult<()> {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()?;

        let schema_variant_1_id = graph.generate_ulid()?;
        let schema_variant_2_id = graph.generate_ulid()?;

        let sv_1 = NodeWeight::new_content(
            schema_variant_1_id,
            schema_variant_1_id,
            ContentAddress::SchemaVariant(ContentHash::new(
                &schema_variant_1_id.inner().to_bytes(),
            )),
        );

        let sv_2 = NodeWeight::new_content(
            schema_variant_2_id,
            schema_variant_2_id,
            ContentAddress::SchemaVariant(ContentHash::new(
                &schema_variant_2_id.inner().to_bytes(),
            )),
        );

        let component_id = graph.generate_ulid()?;
        let component = NodeWeight::new_component(
            component_id,
            component_id,
            ContentHash::new(&component_id.inner().to_bytes()),
        );

        let sv_1_idx = graph.add_or_replace_node(sv_1.clone())?;

        graph.add_edge(
            graph.root(),
            EdgeWeight::new(EdgeWeightKind::new_use()),
            sv_1_idx,
        )?;

        let component_idx = graph.add_or_replace_node(component.clone())?;

        graph.add_edge(
            graph.root(),
            EdgeWeight::new(EdgeWeightKind::new_use()),
            component_idx,
        )?;

        let root_node = graph.get_node_weight(graph.root())?.to_owned();

        let updates = vec![
            Update::NewNode {
                node_weight: sv_2.clone(),
            },
            Update::NewEdge {
                source: (&root_node).into(),
                destination: (&sv_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
        ];
        let new_updates = component.correct_exclusive_outgoing_edges(&graph, updates.clone());
        // There are no exclusive edges here so we should not produce an update
        assert_eq!(
            new_updates, updates,
            "no exclusive edges, therefore no new updates"
        );

        let updates = vec![
            Update::NewNode {
                node_weight: sv_2.clone(),
            },
            Update::NewEdge {
                source: (&root_node).into(),
                destination: (&sv_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
            Update::NewEdge {
                source: (&component).into(),
                destination: (&sv_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
        ];

        let new_updates = component.correct_exclusive_outgoing_edges(&graph, updates.clone());
        // nothing should change here, either, since we haven't added the exclusive edge
        assert_eq!(
            new_updates, updates,
            "no exclusive edge already in the graph, so no new updates"
        );

        // this is the "exclusive" edge
        graph.add_edge(
            graph.get_node_index_by_id(component_id)?,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            graph.get_node_index_by_id(schema_variant_1_id)?,
        )?;

        let updates = vec![
            Update::NewNode {
                node_weight: sv_2.clone(),
            },
            Update::NewEdge {
                source: (&root_node).into(),
                destination: (&sv_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
            Update::NewEdge {
                source: (&component).into(),
                destination: (&sv_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
        ];

        let new_updates = component.correct_exclusive_outgoing_edges(&graph, updates.clone());
        let mut expected_updates = updates.clone();
        let expected_remove_edge_update = Update::RemoveEdge {
            source: (&component).into(),
            destination: (&sv_1).into(),
            edge_kind: EdgeWeightKindDiscriminants::Use,
        };
        expected_updates.push(expected_remove_edge_update.clone());

        assert_eq!(
            expected_updates, new_updates,
            "has a new remove edge update"
        );

        let updates = vec![
            Update::NewNode {
                node_weight: sv_2.clone(),
            },
            Update::NewEdge {
                source: (&root_node).into(),
                destination: (&sv_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
            Update::NewEdge {
                source: (&component).into(),
                destination: (&sv_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
            expected_remove_edge_update.clone(),
        ];

        let new_updates = component.correct_exclusive_outgoing_edges(&graph, updates.clone());
        // with the remove edge already there, there should be no change
        assert_eq!(
            updates, new_updates,
            "remove edge in the set of updates, so no new update"
        );

        Ok(())
    }

    #[test]
    fn correct_exclusive_outgoing_action_edges() -> WorkspaceSnapshotGraphResult<()> {
        let mut graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()?;

        let action_id = graph.generate_ulid()?;
        let prototype_1_id = graph.generate_ulid()?;
        let prototype_2_id = graph.generate_ulid()?;
        let component_1_id = graph.generate_ulid()?;
        let component_2_id = graph.generate_ulid()?;

        let action = NodeWeight::new_action(Ulid::new().into(), action_id, action_id);
        graph.add_or_replace_node(action.clone())?;

        let prototype_1 = NodeWeight::new_action_prototype(
            prototype_1_id,
            prototype_1_id,
            ActionKind::Create,
            "create".into(),
            None,
        );

        let prototype_2 = NodeWeight::new_action_prototype(
            prototype_2_id,
            prototype_2_id,
            ActionKind::Create,
            "create 2".into(),
            None,
        );

        let component_1 = NodeWeight::new_component(
            component_1_id,
            component_1_id,
            ContentHash::new(&component_1_id.inner().to_bytes()),
        );

        let component_2 = NodeWeight::new_component(
            component_2_id,
            component_2_id,
            ContentHash::new(&component_2_id.inner().to_bytes()),
        );

        let prototype_1_idx = graph.add_or_replace_node(prototype_1.clone())?;

        graph.add_edge(
            graph.root(),
            EdgeWeight::new(EdgeWeightKind::new_use()),
            graph.get_node_index_by_id(action.id())?,
        )?;

        graph.add_edge(
            graph.get_node_index_by_id(action.id())?,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            prototype_1_idx,
        )?;

        let root_node = graph.get_node_weight(graph.root())?.to_owned();

        let updates = vec![
            Update::NewNode {
                node_weight: prototype_2.clone(),
            },
            Update::NewEdge {
                source: (&root_node).into(),
                destination: (&prototype_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
        ];
        let new_updates = action
            .correct_transforms(&graph, updates.clone(), false)
            .expect("correct transforms");
        // There are no exclusive edges here so we should not produce an update
        assert_eq!(
            new_updates, updates,
            "no exclusive edges, therefore no new updates"
        );
        let updates = vec![
            Update::NewNode {
                node_weight: prototype_2.clone(),
            },
            Update::NewEdge {
                source: (&action).into(),
                destination: (&prototype_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
        ];
        let new_updates = action
            .correct_transforms(&graph, updates.clone(), false)
            .expect("correct transforms");
        let new_remove_edge = Update::RemoveEdge {
            source: (&action).into(),
            destination: (&prototype_1).into(),
            edge_kind: EdgeWeightKindDiscriminants::Use,
        };
        assert_eq!(
            new_remove_edge,
            new_updates
                .last()
                .expect("there should be a last one!")
                .to_owned()
        );

        let updates = vec![
            Update::NewNode {
                node_weight: component_1.clone(),
            },
            Update::NewEdge {
                source: (&action).into(),
                destination: (&component_1).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
        ];
        let new_updates = action
            .correct_transforms(&graph, updates.clone(), false)
            .expect("correct transforms");
        // There are no exclusive edges here so we should not produce an update
        assert_eq!(
            new_updates, updates,
            "no conflict with the component edges, therefore no new updates"
        );
        graph.perform_updates(&new_updates)?;

        let updates = vec![
            Update::NewNode {
                node_weight: component_2.clone(),
            },
            Update::NewNode {
                node_weight: prototype_2.clone(),
            },
            Update::NewEdge {
                source: (&action).into(),
                destination: (&component_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
            Update::NewEdge {
                source: (&action).into(),
                destination: (&prototype_2).into(),
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            },
        ];

        let mut new_updates = action
            .correct_transforms(&graph, updates.clone(), false)
            .expect("correct transforms");
        let new_remove_edge_prototype = Update::RemoveEdge {
            source: (&action).into(),
            destination: (&prototype_1).into(),
            edge_kind: EdgeWeightKindDiscriminants::Use,
        };
        let new_remove_edge_component = Update::RemoveEdge {
            source: (&action).into(),
            destination: (&component_1).into(),
            edge_kind: EdgeWeightKindDiscriminants::Use,
        };

        assert_eq!(2, new_updates.len() - updates.len());

        let new_update_1 = new_updates.pop().expect("should exist");
        let new_update_2 = new_updates.pop().expect("should exist");

        for new_update in [new_update_1, new_update_2] {
            assert!(matches!(new_update, Update::RemoveEdge { .. }));
            if let Update::RemoveEdge { destination, .. } = &new_update {
                if destination.node_weight_kind == NodeWeightDiscriminants::Component {
                    assert_eq!(new_update, new_remove_edge_component);
                } else {
                    assert_eq!(new_update, new_remove_edge_prototype);
                }
            } else {
                unreachable!("we already asserted this above");
            }
        }
        Ok(())
    }
}
