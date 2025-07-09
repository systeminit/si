use async_trait::async_trait;
use si_id::{
    AttributePrototypeArgumentId,
    AttributePrototypeId,
    FuncId,
};

use crate::{
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    attribute::prototype::{
        AttributePrototypeError,
        AttributePrototypeResult,
    },
    workspace_snapshot::{
        split_snapshot::SplitSnapshot,
        traits::attribute_prototype::AttributePrototypeExt,
    },
};

#[async_trait]
impl AttributePrototypeExt for SplitSnapshot {
    async fn attribute_prototype_arguments(
        &self,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeArgumentId>> {
        Ok(self
            .outgoing_targets_for_edge_weight_kind(
                attribute_prototype_id,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )
            .await?
            .iter()
            .copied()
            .map(Into::into)
            .collect())
    }

    async fn attribute_prototype_func_id(
        &self,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<FuncId> {
        let mut funcs = Vec::new();
        for use_target in self
            .outgoing_targets_for_edge_weight_kind(
                attribute_prototype_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            if NodeWeightDiscriminants::Func == self.get_node_weight(use_target).await?.into() {
                funcs.push(use_target);
            }
        }

        match (funcs.pop(), funcs.pop()) {
            (Some(func_id), None) => Ok(func_id.into()),
            (None, None) => Err(AttributePrototypeError::MissingFunction(
                attribute_prototype_id,
            )),
            (Some(_), Some(_)) => Err(AttributePrototypeError::MultipleFunctionsFound(
                attribute_prototype_id,
            )),
            (None, Some(_)) => unreachable!("Vec::pop() had None then Some"),
        }
    }
}
