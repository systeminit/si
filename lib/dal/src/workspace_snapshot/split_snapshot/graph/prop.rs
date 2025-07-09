use si_id::PropId;

use crate::{
    prop::PropResult,
    workspace_snapshot::{
        graph::traits::prop::PropExt,
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
}
