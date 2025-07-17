use si_id::PropId;

use crate::prop::PropResult;

pub trait PropExt {
    fn ordered_child_prop_ids(&self, prop_id: PropId) -> PropResult<Vec<PropId>>;
}
