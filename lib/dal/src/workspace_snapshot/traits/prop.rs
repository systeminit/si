use async_trait::async_trait;
use petgraph::prelude::*;
use si_id::PropId;

use crate::{
    prop::PropResult, workspace_snapshot::node_weight::PropNodeWeight, EdgeWeightKindDiscriminants,
    PropKind, WorkspaceSnapshot, WorkspaceSnapshotGraphVCurrent,
};

#[async_trait]
pub trait PropExt {
    /// Generate a TypeScript type for a prop tree.
    async fn ts_type(&self, prop_id: PropId) -> PropResult<String>;
}

#[async_trait]
impl PropExt for WorkspaceSnapshot {
    async fn ts_type(&self, prop_id: PropId) -> PropResult<String> {
        let graph = self.working_copy().await;
        let index = graph.get_node_index_by_id(prop_id)?;
        let node = graph.get_node_weight(index)?.as_prop_node_weight()?;
        let mut result = String::new();
        append_ts_type(&graph, node, index, &mut result)?;
        Ok(result)
    }
}

fn append_ts_type(
    graph: &WorkspaceSnapshotGraphVCurrent,
    node: &PropNodeWeight,
    index: NodeIndex,
    buf: &mut String,
) -> PropResult<()> {
    /// Check if the parent of the given node has the specified path.
    fn parent_has_path(
        graph: &WorkspaceSnapshotGraphVCurrent,
        index: NodeIndex,
        path: &[&str],
    ) -> PropResult<bool> {
        // Get the parent
        let parent_index = graph.get_edge_weight_kind_target_idx_opt(
            index,
            Incoming,
            EdgeWeightKindDiscriminants::Use,
        )?;

        // If the path is empty, we match iff there is no parent
        let Some((&name, parent_path)) = path.split_last() else {
            return Ok(parent_index.is_none());
        };

        // If the path is non-empty, but we have a parent, we don't match
        let Some(parent_index) = parent_index else {
            return Ok(false);
        };

        let node = graph.get_node_weight(parent_index)?.as_prop_node_weight()?;
        Ok(name == node.name() && parent_has_path(graph, parent_index, parent_path)?)
    }

    // Special cases
    if node.name() == "status" && parent_has_path(graph, index, &["root", "resource"])? {
        buf.push_str("'ok' | 'warning' | 'error' | undefined | null");
        return Ok(());
    }
    if node.name() == "payload" && parent_has_path(graph, index, &["root", "resource"])? {
        buf.push_str("any");
        return Ok(());
    }

    match node.kind() {
        PropKind::Array => {
            append_ts_element_type(graph, index, buf)?;
            buf.push_str("[]");
        }
        PropKind::Boolean => buf.push_str("boolean"),
        PropKind::Float | PropKind::Integer => buf.push_str("number"),
        PropKind::Json => buf.push_str("any"),
        PropKind::Map => {
            buf.push_str("Record<string, ");
            append_ts_element_type(graph, index, buf)?;
            buf.push('>');
        }
        PropKind::Object => {
            buf.push_str("{\n");
            for child_index in graph.ordered_children_for_node(index)?.unwrap_or(vec![]) {
                let child_node = graph.get_node_weight(child_index)?.as_prop_node_weight()?;
                buf.push_str(&serde_json::to_string(child_node.name())?);
                buf.push_str("?: ");
                append_ts_type(graph, child_node, child_index, buf)?;
                buf.push_str(" | null;\n");
            }
            buf.push('}');
        }
        PropKind::String => buf.push_str("string"),
    };
    Ok(())
}

/// Generate a TypeScript type for the element type of an array or map.
fn append_ts_element_type(
    graph: &WorkspaceSnapshotGraphVCurrent,
    parent_index: NodeIndex,
    buf: &mut String,
) -> PropResult<()> {
    let element_prop_index = graph.get_edge_weight_kind_target_idx(
        parent_index,
        Outgoing,
        EdgeWeightKindDiscriminants::Use,
    )?;
    let element_prop_node = graph
        .get_node_weight(element_prop_index)?
        .as_prop_node_weight()?;
    append_ts_type(graph, element_prop_node, element_prop_index, buf)
}
