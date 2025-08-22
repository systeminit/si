use std::collections::HashMap;

use si_id::PropId;

use crate::{
    prop::PropResult,
    workspace_snapshot::{
        graph::traits::prop::{
            PropExt,
            PropGraphData,
            PropSchemaTreeData,
        },
        split_snapshot::SplitSnapshotGraphV1,
    },
};

impl PropExt for SplitSnapshotGraphV1 {
    fn ordered_child_prop_ids(&self, prop_id: PropId) -> PropResult<Vec<PropId>> {
        Ok(self
            .ordered_children(prop_id.into())
            .unwrap_or_default()
            .iter()
            .copied()
            .map(Into::into)
            .collect())
    }

    fn build_prop_schema_tree_data(
        &self,
        root_prop_id: PropId,
    ) -> PropResult<Option<PropSchemaTreeData>> {
        let mut props = HashMap::new();
        let mut children = HashMap::new();

        let mut stack = vec![root_prop_id];
        let mut visited = std::collections::HashSet::new();

        while let Some(prop_id) = stack.pop() {
            if visited.contains(&prop_id) {
                continue;
            }
            visited.insert(prop_id);

            let prop_node_weight = self
                .node_weight(prop_id.into())
                .ok_or_else(|| crate::prop::PropError::PropNotFound(prop_id))?
                .get_prop_node_weight()?;

            let prop_data = PropGraphData {
                id: prop_id,
                name: prop_node_weight.name().to_string(),
                kind: prop_node_weight.kind(),
                content_hash: prop_node_weight.content_hash(),
            };
            props.insert(prop_id, prop_data);

            let child_ids = self.ordered_child_prop_ids(prop_id)?;
            if !child_ids.is_empty() {
                children.insert(prop_id, child_ids.clone());
                stack.extend(child_ids);
            }
        }

        Ok(Some(PropSchemaTreeData {
            props,
            children,
            root_id: root_prop_id,
        }))
    }
}
