use crate::workspace_snapshot::graph::{
    WorkspaceSnapshotGraphError,
    WorkspaceSnapshotGraphV4,
    traits::prop::PropExt,
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
}
