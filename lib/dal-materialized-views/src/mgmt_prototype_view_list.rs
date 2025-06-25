use dal::{
    DalContext,
    Func,
    SchemaVariantId,
    management::prototype::ManagementPrototype,
};
use si_frontend_mv_types::management::{
    ManagementFuncKind,
    MgmtPrototypeView,
};
use telemetry::prelude::*;

#[instrument(
  name = "dal_materialized_views.mgmt_prototype_view_list"
  level = "debug",
  skip_all
)]
pub async fn assemble(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> super::Result<Vec<MgmtPrototypeView>> {
    let ctx = &ctx;
    let mut views = Vec::new();

    for p in ManagementPrototype::list_for_variant_id(ctx, schema_variant_id).await? {
        let func_id = ManagementPrototype::func_id(ctx, p.id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;
        // TODO: Make Management Func Kinds a real thing
        let kind = {
            if func.name == *"Import from AWS" {
                ManagementFuncKind::Import
            } else if func.name == *"Discover on AWS" {
                ManagementFuncKind::Discover
            } else {
                ManagementFuncKind::Other
            }
        };

        let name = if let Some(display_name) = func.display_name {
            display_name
        } else {
            func.name
        };

        views.push(MgmtPrototypeView {
            id: p.id,
            func_id,
            description: p.description,
            prototype_name: p.name,
            name,
            kind,
        });
    }

    views.sort_by_key(|v| v.id);
    Ok(views)
}
