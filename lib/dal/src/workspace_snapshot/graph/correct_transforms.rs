use std::collections::{HashMap, HashSet};

use crate::workspace_snapshot::node_weight::traits::{CorrectTransforms, CorrectTransformsResult};

use super::{detect_updates::Update, WorkspaceSnapshotGraphV2};

pub fn correct_transforms(
    graph: &WorkspaceSnapshotGraphV2,
    mut updates: Vec<Update>,
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
            updates = node_weight.correct_transforms(graph, updates)?;
        }
    }

    Ok(updates)
}
