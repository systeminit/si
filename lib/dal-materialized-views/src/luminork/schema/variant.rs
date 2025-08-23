use dal::{
    DalContext,
    Prop,
    SchemaVariant,
    SchemaVariantId,
    schema::variant::root_prop::RootPropChild,
    workspace_snapshot::traits::prop::PropExt,
};
use si_frontend_mv_types::luminork_schema_variant::LuminorkSchemaVariant as LuminorkSchemaVariantMv;
use telemetry::prelude::*;

pub mod default;

#[instrument(
    name = "dal_materialized_views.luminork.schema.variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    id: SchemaVariantId,
) -> crate::Result<LuminorkSchemaVariantMv> {
    let schema_variant = SchemaVariant::get_by_id(&ctx, id).await?;

    let variant_func_ids: Vec<_> = SchemaVariant::all_func_ids(&ctx, id)
        .await?
        .into_iter()
        .collect();

    let domain_props = {
        let domain = Prop::find_prop_by_path(&ctx, id, &RootPropChild::Domain.prop_path()).await?;

        let prop_schema_tree = ctx
            .workspace_snapshot()?
            .build_prop_schema_tree(&ctx, domain.id)
            .await?;

        Some(prop_schema_tree)
    };

    let is_default_variant = SchemaVariant::is_default_by_id(&ctx, id).await?;
    let asset_func_id = schema_variant.asset_func_id_or_error()?;

    Ok(LuminorkSchemaVariantMv::new(
        id,
        schema_variant.display_name().into(),
        schema_variant.category().into(),
        schema_variant.color().into(),
        schema_variant.is_locked(),
        schema_variant.description(),
        schema_variant.link(),
        asset_func_id,
        variant_func_ids,
        is_default_variant,
        domain_props,
    ))
}
