use async_trait::async_trait;
use si_id::{
    AttributePrototypeId,
    PropId,
};

use crate::{
    DalContext,
    EdgeWeightKindDiscriminants,
    prop::{
        PropError,
        PropResult,
    },
    workspace_snapshot::{
        split_snapshot::SplitSnapshot,
        traits::{
            attribute_prototype::AttributePrototypeExt as _,
            attribute_prototype_argument::AttributePrototypeArgumentExt as _,
            func::FuncExt as _,
            prop::PropExt,
        },
    },
};

#[async_trait]
impl PropExt for SplitSnapshot {
    async fn prop_default_value(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Option<serde_json::Value>> {
        let prototype_id = self.prop_prototype_id(prop_id).await?;
        let func_id = self.attribute_prototype_func_id(prototype_id).await?;
        if self.func_is_dynamic(func_id).await? {
            return Ok(None);
        }

        match self
            .attribute_prototype_arguments(prototype_id)
            .await?
            .first()
        {
            Some(&apa_id) => self
                .attribute_prototype_argument_static_value(ctx, apa_id)
                .await
                .map_err(Into::into),
            None => Ok(None),
        }
    }

    async fn prop_prototype_id(&self, prop_id: PropId) -> PropResult<AttributePrototypeId> {
        self.outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Prototype)
            .await?
            .first()
            .copied()
            .map(Into::into)
            .ok_or_else(|| PropError::MissingPrototypeForProp(prop_id))
    }

    async fn ts_type(&self, _prop_id: PropId) -> PropResult<String> {
        Ok("any".to_string())
    }
}
