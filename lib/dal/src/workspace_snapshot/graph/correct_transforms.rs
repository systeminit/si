use std::collections::{
    HashMap,
    HashSet,
};

use petgraph::prelude::*;
use si_events::ulid::Ulid;

use super::{
    WorkspaceSnapshotGraphVCurrent,
    detector::Update,
};
use crate::{
    EdgeWeight,
    EdgeWeightKind,
    NodeWeightDiscriminants,
    workspace_snapshot::{
        NodeInformation,
        node_weight::{
            NodeWeight,
            category_node_weight::CategoryNodeKind,
            traits::{
                CorrectTransforms,
                CorrectTransformsResult,
            },
        },
    },
};

pub fn correct_transforms(
    graph: &WorkspaceSnapshotGraphVCurrent,
    mut updates: Vec<Update>,
    from_different_change_set: bool,
) -> CorrectTransformsResult<Vec<Update>> {
    let mut new_nodes = HashMap::new();
    let mut nodes_to_interrogate = HashSet::new();

    for update in &updates {
        match update {
            Update::NewEdge {
                source,
                destination,
                ..
            } => {
                nodes_to_interrogate.insert(source.id);
                nodes_to_interrogate.insert(destination.id);
            }
            Update::RemoveEdge {
                source,
                destination,
                ..
            } => {
                nodes_to_interrogate.insert(source.id);
                nodes_to_interrogate.insert(destination.id);
            }
            Update::NewNode { node_weight } => {
                new_nodes.insert(node_weight.id(), node_weight.clone());
                nodes_to_interrogate.insert(node_weight.id().into());
            }
            Update::ReplaceNode { node_weight } => {
                nodes_to_interrogate.insert(node_weight.id().into());
            }
        }
    }

    //
    // Let each node involved in the updates check for / resolve conflicts.
    //
    for node_to_interrogate in nodes_to_interrogate {
        // If the node weight isn't in the graph, see if it was in a NewNode update and
        // pass that node weight.
        let node_index = graph.get_node_index_by_id_opt(node_to_interrogate);
        if let Some(node_weight) = match node_index {
            Some(node_index) => graph.get_node_weight_opt(node_index),
            None => new_nodes.get(&node_to_interrogate.into()),
        } {
            updates = node_weight.correct_transforms(graph, updates, from_different_change_set)?;
        }
    }

    Ok(updates)
}

/// Produce the NewNode and NewEdge updates required for adding a dependent value root to the graph
pub fn add_dependent_value_root_updates(
    graph: &WorkspaceSnapshotGraphVCurrent,
    value_ids: &HashSet<Ulid>,
) -> CorrectTransformsResult<Vec<Update>> {
    let mut updates = vec![];

    if let Some((category_node_id, category_node_idx)) =
        graph.get_category_node(CategoryNodeKind::DependentValueRoots)?
    {
        let existing_dvu_nodes: Vec<_> = graph
            .edges_directed(category_node_idx, Outgoing)
            .filter_map(|edge_ref| {
                graph
                    .get_node_weight_opt(edge_ref.target())
                    .and_then(|weight| match weight {
                        NodeWeight::DependentValueRoot(inner) => Some(inner.value_id()),
                        _ => None,
                    })
            })
            .collect();

        for value_id in value_ids {
            if existing_dvu_nodes.contains(value_id) {
                continue;
            }

            let id = graph.generate_ulid()?;
            let lineage_id = graph.generate_ulid()?;
            let new_dvu_node = NodeWeight::new_dependent_value_root(id, lineage_id, *value_id);
            let new_dvu_node_information = (&new_dvu_node).into();

            updates.push(Update::NewNode {
                node_weight: new_dvu_node,
            });
            updates.push(Update::NewEdge {
                source: NodeInformation {
                    id: category_node_id.into(),
                    node_weight_kind: NodeWeightDiscriminants::Category,
                },
                destination: new_dvu_node_information,
                edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
            });
        }
    }

    Ok(updates)
}
