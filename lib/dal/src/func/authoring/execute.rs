use telemetry::prelude::*;

use crate::func::authoring::FuncAuthoringResult;
use crate::{AttributePrototype, AttributeValue, DalContext, Func};

#[instrument(
    name = "func.authoring.execute_func.execute_attribute_func",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn execute_attribute_func(
    ctx: &DalContext,
    func: &Func,
) -> FuncAuthoringResult<()> {
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func.id).await?;

    if attribute_prototype_ids.is_empty() {
        warn!(%func.id, "nothing to execute: no attribute prototype ids found for attribute func id");
        return Ok(());
    }

    for attribute_prototype_id in attribute_prototype_ids {
        let attribute_value_ids =
            AttributePrototype::attribute_value_ids(ctx, attribute_prototype_id).await?;

        for attribute_value_id in &attribute_value_ids {
            AttributeValue::update_from_prototype_function(ctx, *attribute_value_id).await?;
        }

        ctx.add_dependent_values_and_enqueue(attribute_value_ids)
            .await?;
    }

    Ok(())
}
