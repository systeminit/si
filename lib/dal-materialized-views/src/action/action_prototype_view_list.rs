use dal::{
    DalContext,
    Func,
    SchemaVariantId,
    action::prototype::ActionPrototype,
};
use si_frontend_mv_types::action::{
    ActionPrototypeView,
    ActionPrototypeViewList as ActionPrototypeViewListMv,
};
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.action_prototype_view_list"
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    schema_variant_id: SchemaVariantId,
) -> crate::Result<ActionPrototypeViewListMv> {
    let ctx = &ctx;

    let action_prototypes_for_variant =
        ActionPrototype::for_variant(ctx, schema_variant_id).await?;
    let mut action_prototypes = Vec::with_capacity(action_prototypes_for_variant.len());

    for action_prototype in action_prototypes_for_variant {
        let func_id = ActionPrototype::func_id(ctx, action_prototype.id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;

        action_prototypes.push(ActionPrototypeView {
            id: action_prototype.id,
            func_id,
            kind: action_prototype.kind.into(),
            display_name: func.display_name,
            name: func.name,
        });
    }

    Ok(ActionPrototypeViewListMv {
        id: schema_variant_id,
        action_prototypes,
    })
}
