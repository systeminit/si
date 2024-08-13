#[allow(clippy::panic)]
#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
mod test {
    use si_events::ContentHash;

    use crate::{
        workspace_snapshot::{
            content_address::ContentAddress,
            graph::{detect_updates::Update, WorkspaceSnapshotGraphResult},
            node_weight::{traits::CorrectExclusiveOutgoingEdge, NodeWeight},
        },
        EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants, WorkspaceSnapshotGraphV2,
    };

    #[test]
    fn correct_exclusive_outgoing_edges() -> WorkspaceSnapshotGraphResult<()> {
        let mut graph = WorkspaceSnapshotGraphV2::new()?;

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

        let sv_1_idx = graph.add_node(sv_1.clone())?;

        graph.add_edge(
            graph.root(),
            EdgeWeight::new(EdgeWeightKind::new_use()),
            sv_1_idx,
        )?;

        let component_idx = graph.add_node(component.clone())?;

        graph.add_edge(
            graph.root_index,
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
}
