use std::collections::BTreeMap;

use petgraph::visit::{
    EdgeRef,
    IntoEdgeReferences,
};
use si_events::WorkspaceSnapshotAddress;
use telemetry::prelude::*;

use super::SnapshotGraphMigratorResult;
use crate::{
    DalContext,
    EdgeWeightKind,
    workspace_snapshot::{
        graph::WorkspaceSnapshotGraphV4,
        node_weight::NodeWeight,
        split_snapshot::{
            SplitSnapshot,
            SplitSnapshotGraph,
            SplitSnapshotGraphV1,
        },
    },
};

#[instrument(skip_all)]
pub async fn migrate_v4_to_split_v1(
    ctx: &DalContext,
    v4_graph: WorkspaceSnapshotGraphV4,
) -> SnapshotGraphMigratorResult<WorkspaceSnapshotAddress> {
    let mut split_graph = SplitSnapshotGraphV1::new(usize::MAX / 2);

    let mut node_id_by_index = BTreeMap::new();
    let mut ordering_for_node = BTreeMap::new();
    let root_node_index = v4_graph.root();
    let root_node_id = v4_graph.get_node_weight(root_node_index)?.id();
    let split_root_id = split_graph.root_id()?;

    for node_index in v4_graph.graph().node_indices() {
        let Some(node) = v4_graph.graph().node_weight(node_index).cloned() else {
            continue;
        };

        let node_id = node.id();
        if node_id == root_node_id {
            continue;
        }
        if let NodeWeight::Ordering(_) = node {
            continue;
        }

        node_id_by_index.insert(node_index, node_id);
        split_graph.add_or_replace_node(node)?;

        if let Some(ordering_node) = v4_graph.ordering_node_for_container(node_index)? {
            let order = ordering_node.order().clone();
            ordering_for_node.insert(node_id, order);
            // Ordering nodes are an internal implementation detail for the split graph,
            // but we still need to force the creation of them for all "ordered" nodes,
            // since the DAL expects them to exist even if there are no ordered children
            split_graph.add_ordering_node_for_node_id(node_id)?;
        }
    }

    for edge_ref in v4_graph.graph().edge_references() {
        if matches!(
            edge_ref.weight().kind(),
            EdgeWeightKind::Ordinal | EdgeWeightKind::Ordering
        ) {
            continue;
        }

        let source_idx = edge_ref.source();
        let target_idx = edge_ref.target();
        let edge_weight = edge_ref.weight().clone();

        let source_id = if source_idx == root_node_index {
            split_root_id
        } else {
            v4_graph.get_node_weight(source_idx)?.id()
        };

        let target_id = v4_graph.get_node_weight(target_idx)?.id();

        match ordering_for_node.get(&source_id) {
            Some(order) => {
                if order.contains(&target_id) {
                    split_graph.add_ordered_edge(source_id, edge_weight, target_id)?;
                } else {
                    split_graph.add_edge(source_id, edge_weight, target_id)?;
                }
            }
            None => {
                split_graph.add_edge(source_id, edge_weight, target_id)?;
            }
        }
    }

    for (node_id, ordering) in ordering_for_node {
        split_graph.reorder_node(node_id, move |_| ordering.clone())?;
    }

    let split_snapshot = SplitSnapshot::from_graph(SplitSnapshotGraph::V1(split_graph));

    Ok(split_snapshot.write(ctx).await?)
}
