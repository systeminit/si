use async_trait::async_trait;
use si_id::{
    AttributePrototypeArgumentId,
    AttributePrototypeId,
    FuncId,
};

use crate::{
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    WorkspaceSnapshot,
    attribute::prototype::{
        AttributePrototypeError,
        AttributePrototypeResult,
    },
};

#[async_trait]
pub trait AttributePrototypeExt {
    /// All [`AttributePrototypeArgumentId`] for the given [`AttributePrototypeId`].
    async fn attribute_prototype_arguments(
        &self,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeArgumentId>>;

    /// The [`FuncId`] for the given [`AttributePrototypeId`].
    async fn attribute_prototype_func_id(
        &self,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<FuncId>;
}

#[async_trait]
impl AttributePrototypeExt for WorkspaceSnapshot {
    async fn attribute_prototype_arguments(
        &self,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeArgumentId>> {
        let mut apa_ids: Vec<_> = self
            .outgoing_targets_for_edge_weight_kind(
                attribute_prototype_id,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )
            .await?
            .iter()
            .copied()
            .map(Into::into)
            .collect();
        // Whenever we're dealing with the arguments, we always want them to have stable
        // ordering. For function execution, we want the arguments to have the same relative
        // order from run to run. For use when generating checksums and other lists we also
        // want them to have consistent order to avoid unnecessary churn in the data sent to
        // & used by the front end.
        apa_ids.sort();

        Ok(apa_ids)
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
