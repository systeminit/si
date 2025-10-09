use dal::{
    DalContext,
    Func,
    SchemaVariantId,
    management::prototype::{
        ManagementFuncKind as dalMgmtFuncKind,
        ManagementPrototype,
    },
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

    let management_prototypes_for_variant =
        ManagementPrototype::list_for_schema_and_variant_id(ctx, schema_variant_id).await?;
    let mut views = Vec::with_capacity(management_prototypes_for_variant.len());

    for p in management_prototypes_for_variant {
        let func_id = ManagementPrototype::func_id(ctx, p.id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;
        let kind = ManagementPrototype::kind_by_id(ctx, p.id).await?;

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
            kind: convert_kind(kind),
        });
    }

    views.sort_by_key(|v| v.id);
    Ok(views)
}

fn convert_kind(kind: dalMgmtFuncKind) -> ManagementFuncKind {
    match kind {
        dalMgmtFuncKind::Discover => ManagementFuncKind::Discover,
        dalMgmtFuncKind::Import => ManagementFuncKind::Import,
        dalMgmtFuncKind::Other => ManagementFuncKind::Other,
        dalMgmtFuncKind::RunTemplate => ManagementFuncKind::RunTemplate,
    }
}
