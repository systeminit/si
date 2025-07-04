use si_id::{
    AttributePrototypeId,
    AttributeValueId,
};

use crate::{
    EdgeWeightKindDiscriminants,
    attribute::value::{
        AttributeValueError,
        AttributeValueResult,
    },
    workspace_snapshot::graph::{
        WorkspaceSnapshotGraphV4,
        traits::attribute_value::AttributeValueExt,
    },
};

impl AttributeValueExt for WorkspaceSnapshotGraphV4 {
    fn component_prototype_id(
        &self,
        id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>> {
        let node_idx = self
            .get_node_index_by_id(id)
            .map_err(|_| AttributeValueError::MissingForId(id))?;
        let maybe_prototype_idx = self
            .target_opt(node_idx, EdgeWeightKindDiscriminants::Prototype)
            .map_err(|_| AttributeValueError::MultiplePrototypesFound(id))?
            .and_then(|node_idx| self.node_index_to_id(node_idx).map(Into::into));

        Ok(maybe_prototype_idx)
    }
}
