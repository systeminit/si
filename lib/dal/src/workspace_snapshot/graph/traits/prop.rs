use std::collections::HashMap;

use si_events::ContentHash;
use si_id::PropId;

use crate::{
    PropKind,
    prop::PropResult,
};

#[derive(Debug, Clone)]
pub struct PropSchemaTreeData {
    pub props: HashMap<PropId, PropGraphData>,
    pub children: HashMap<PropId, Vec<PropId>>,
    pub root_id: PropId,
}

#[derive(Debug, Clone)]
pub struct PropGraphData {
    pub id: PropId,
    pub name: String,
    pub kind: PropKind,
    pub content_hash: ContentHash,
}

pub trait PropExt {
    fn ordered_child_prop_ids(&self, prop_id: PropId) -> PropResult<Vec<PropId>>;
    fn build_prop_schema_tree_data(
        &self,
        root_prop_id: PropId,
    ) -> PropResult<Option<PropSchemaTreeData>>;
}
