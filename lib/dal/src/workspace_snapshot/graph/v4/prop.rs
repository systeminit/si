use std::collections::HashMap;

use petgraph::{
    prelude::*,
    visit::{
        Control,
        DfsEvent,
    },
};

use crate::workspace_snapshot::graph::{
    WorkspaceSnapshotGraphError,
    WorkspaceSnapshotGraphResult,
    WorkspaceSnapshotGraphV4,
    traits::prop::{
        PropExt,
        PropGraphData,
        PropSchemaTreeData,
    },
};

impl PropExt for WorkspaceSnapshotGraphV4 {
    fn ordered_child_prop_ids(
        &self,
        prop_id: si_id::PropId,
    ) -> crate::prop::PropResult<Vec<si_id::PropId>> {
        let prop_idx = self.get_node_index_by_id(prop_id)?;
        let child_prop_idxs = self
            .ordered_children_for_node(prop_idx)?
            .unwrap_or_default();
        let mut child_prop_ids = Vec::with_capacity(child_prop_idxs.len());
        for child_prop_idx in child_prop_idxs {
            child_prop_ids.push(
                self.node_index_to_id(child_prop_idx)
                    .ok_or_else(|| {
                        WorkspaceSnapshotGraphError::NodeWithIndexNotFound(child_prop_idx)
                    })?
                    .into(),
            );
        }

        Ok(child_prop_ids)
    }

    fn build_prop_schema_tree_data(
        &self,
        root_prop_id: si_id::PropId,
    ) -> crate::prop::PropResult<Option<PropSchemaTreeData>> {
        // Check if root prop exists - return None if missing (not an error)
        let root_idx = match self.get_node_index_by_id(root_prop_id) {
            Ok(idx) => idx,
            Err(WorkspaceSnapshotGraphError::NodeWithIdNotFound(_)) => return Ok(None),
            Err(e) => return Err(e.into()), // Propagate other errors
        };

        let mut props = HashMap::new();
        let mut children = HashMap::new();

        // Use DFS to traverse the prop tree - propagate any errors
        petgraph::visit::depth_first_search(&self.graph, Some(root_idx), |event| {
            build_prop_schema_tree_dfs_event(event, &mut props, &mut children, self)
        })?;

        Ok(Some(PropSchemaTreeData {
            props,
            children,
            root_id: root_prop_id,
        }))
    }
}

fn build_prop_schema_tree_dfs_event(
    event: DfsEvent<NodeIndex>,
    props: &mut HashMap<si_id::PropId, PropGraphData>,
    children: &mut HashMap<si_id::PropId, Vec<si_id::PropId>>,
    graph: &WorkspaceSnapshotGraphV4,
) -> WorkspaceSnapshotGraphResult<Control<()>> {
    match event {
        DfsEvent::Discover(node_idx, _) => {
            let node_weight = graph.get_node_weight(node_idx)?;

            let prop_node_weight = match node_weight.get_prop_node_weight() {
                Ok(prop_weight) => prop_weight,
                Err(_) => return Ok(Control::Prune), // Not a prop node, skip this branch
            };

            let prop_id: si_id::PropId = prop_node_weight.id().into();

            let prop_data = PropGraphData {
                id: prop_id,
                name: prop_node_weight.name().to_string(),
                kind: prop_node_weight.kind(),
                content_hash: prop_node_weight.content_hash(),
            };
            props.insert(prop_id, prop_data);

            let child_prop_ids = match graph.ordered_children_for_node(node_idx)? {
                Some(child_indices) => {
                    let mut child_ids = Vec::with_capacity(child_indices.len());
                    for child_idx in child_indices {
                        let child_id = graph.node_index_to_id(child_idx).ok_or_else(|| {
                            WorkspaceSnapshotGraphError::NodeWithIndexNotFound(child_idx)
                        })?;
                        child_ids.push(child_id.into());
                    }
                    child_ids
                }
                None => Vec::new(),
            };

            if !child_prop_ids.is_empty() {
                children.insert(prop_id, child_prop_ids);
            }

            Ok(Control::Continue)
        }
        _ => Ok(Control::Continue),
    }
}
