use telemetry::prelude::warn;

use crate::func::authoring::{FuncAuthoringError, FuncAuthoringResult};
use crate::func::FuncKind;
use crate::{AttributePrototype, AttributeValue, DalContext, Func, FuncId};

pub(crate) async fn execute_func(ctx: &DalContext, id: FuncId) -> FuncAuthoringResult<()> {
    let func = Func::get_by_id_or_error(ctx, id).await?;

    match func.kind {
        FuncKind::Attribute => update_values_for_func(ctx, &func).await?,
        FuncKind::Action => {
            // TODO(nick): fully restore or wait for actions v2. Essentially, we need to run
            // every prototype using the func id for every component.
            warn!("skipping action execution...");
            return Ok(());
        }
        kind => return Err(FuncAuthoringError::NotRunnable(id, kind)),
    };

    Ok(())
}

async fn update_values_for_func(ctx: &DalContext, func: &Func) -> FuncAuthoringResult<()> {
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

        ctx.enqueue_dependent_values_update(attribute_value_ids)
            .await?;
    }

    Ok(())
}
