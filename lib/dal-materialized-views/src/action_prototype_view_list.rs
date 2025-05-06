use dal::{
    DalContext,
    Func,
    SchemaVariantId,
    action::prototype::ActionPrototype,
};
use si_frontend_types::action::{
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
) -> super::Result<ActionPrototypeViewListMv> {
    let ctx = &ctx;
    let mut views = Vec::new();

    for action_prototype in ActionPrototype::for_variant(ctx, schema_variant_id).await? {
        let func_id = ActionPrototype::func_id(ctx, action_prototype.id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;

        views.push(ActionPrototypeView {
            id: action_prototype.id,
            func_id,
            kind: action_prototype.kind.into(),
            display_name: func.display_name,
            name: func.name,
        });
    }

    Ok(ActionPrototypeViewListMv {
        id: schema_variant_id,
        action_prototypes: views,
    })
}
