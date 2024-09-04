use petgraph::{prelude::*, visit::IntoEdgeReferences};
use si_events::ulid::Ulid;
use std::collections::{HashMap, HashSet};
use telemetry::prelude::*;

use crate::{
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::{
            deprecated::DeprecatedWorkspaceSnapshotGraphV1, LineageId, WorkspaceSnapshotGraphV2,
        },
        node_weight::NodeWeight,
    },
    EdgeWeight,
};

use super::SnapshotGraphMigratorResult;

#[instrument(skip_all)]
pub fn migrate_v1_to_v2(
    v1_graph: DeprecatedWorkspaceSnapshotGraphV1,
) -> SnapshotGraphMigratorResult<WorkspaceSnapshotGraphV2> {
    let deprecated_graph_inner = &v1_graph.graph;

    let mut new_graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(
        deprecated_graph_inner.node_count(),
        deprecated_graph_inner.edge_count(),
    );

    let mut node_index_by_id: HashMap<Ulid, NodeIndex> = HashMap::new();
    let mut node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>> = HashMap::new();
    // This is just a place holder until we find the root when iterating the nodes
    let mut root_index = v1_graph.root_index;
    let mut old_graph_idx_to_id = HashMap::new();

    for deprecated_node_weight in deprecated_graph_inner.node_weights() {
        let node_weight: NodeWeight = deprecated_node_weight.clone().into();
        let id = node_weight.id();
        let lineage_id = node_weight.lineage_id();

        let is_root_node = if let NodeWeight::Content(content_node_weight) = &node_weight {
            matches!(content_node_weight.content_address(), ContentAddress::Root)
        } else {
            false
        };

        let new_idx = new_graph.add_node(node_weight);
        if is_root_node {
            root_index = new_idx;
        }

        if let Some(idx) = v1_graph.node_index_by_id.get(&id) {
            old_graph_idx_to_id.insert(idx, id);
        }

        node_index_by_id.insert(id, new_idx);
        node_indices_by_lineage_id
            .entry(lineage_id)
            .and_modify(|index_set| {
                index_set.insert(new_idx);
            })
            .or_insert(HashSet::from([new_idx]));
    }

    for edge_ref in deprecated_graph_inner.edge_references() {
        let deprecated_edge_weight = edge_ref.weight();
        let new_edge_weight: EdgeWeight = deprecated_edge_weight.to_owned().into();

        let source_idx = edge_ref.source();
        let target_idx = edge_ref.target();
        let source_id_in_new_graph = old_graph_idx_to_id.get(&source_idx).copied();
        let target_id_in_new_graph = old_graph_idx_to_id.get(&target_idx).copied();

        if let (Some(source_id), Some(target_id)) = (source_id_in_new_graph, target_id_in_new_graph)
        {
            let source_idx_in_new_graph = node_index_by_id.get(&source_id).copied();
            let target_idx_in_new_graph = node_index_by_id.get(&target_id).copied();

            if let (Some(new_source_idx), Some(new_target_idx)) =
                (source_idx_in_new_graph, target_idx_in_new_graph)
            {
                new_graph.add_edge(new_source_idx, new_target_idx, new_edge_weight);
            }
        }
    }

    let mut new_snapshot_graph = WorkspaceSnapshotGraphV2::new_from_parts(
        new_graph,
        node_index_by_id,
        node_indices_by_lineage_id,
        root_index,
    );

    new_snapshot_graph.recalculate_entire_merkle_tree_hash()?;
    Ok(new_snapshot_graph)
}
