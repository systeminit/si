use async_trait::async_trait;
use si_id::AttributePrototypeArgumentId;

use crate::{
    DalContext,
    EdgeWeightKindDiscriminants,
    attribute::prototype::argument::AttributePrototypeArgumentResult,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::NodeWeight,
        split_snapshot::SplitSnapshot,
        traits::{
            attribute_prototype_argument::AttributePrototypeArgumentExt,
            static_argument_value::StaticArgumentValueExt as _,
        },
    },
};

#[async_trait]
impl AttributePrototypeArgumentExt for SplitSnapshot {
    async fn attribute_prototype_argument_static_value(
        &self,
        ctx: &DalContext,
        attribute_prototype_argument_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Option<serde_json::Value>> {
        for arg_id in self
            .outgoing_targets_for_edge_weight_kind(
                attribute_prototype_argument_id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )
            .await?
        {
            match self.get_node_weight(arg_id).await? {
                NodeWeight::Content(content_node_weight) => {
                    if content_node_weight.content_address_discriminants()
                        == ContentAddressDiscriminants::StaticArgumentValue
                    {
                        return self
                            .static_argument_value(ctx, content_node_weight.id().into())
                            .await
                            .map(Into::into);
                    }
                }
                _ => continue,
            }
        }

        Ok(None)
    }
}
