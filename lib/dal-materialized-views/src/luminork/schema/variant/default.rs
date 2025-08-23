use dal::{
    DalContext,
    Prop,
    Schema,
    SchemaId,
    SchemaVariant,
    schema::variant::root_prop::RootPropChild,
    workspace_snapshot::traits::prop::PropExt,
};
use si_frontend_mv_types::luminork_default_variant::LuminorkDefaultVariant as LuminorkDefaultVariantMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.luminork.schema.variant.default",
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    schema_id: SchemaId,
) -> crate::Result<LuminorkDefaultVariantMv> {
    let default_variant_id = Schema::default_variant_id(&ctx, schema_id).await?;
    let schema_variant = SchemaVariant::get_by_id(&ctx, default_variant_id).await?;

    let variant_func_ids: Vec<_> = SchemaVariant::all_func_ids(&ctx, default_variant_id)
        .await?
        .into_iter()
        .collect();

    let domain_props = {
        let domain =
            Prop::find_prop_by_path(&ctx, default_variant_id, &RootPropChild::Domain.prop_path())
                .await?;

        let prop_schema_tree = ctx
            .workspace_snapshot()?
            .build_prop_schema_tree(&ctx, domain.id)
            .await?;

        Some(prop_schema_tree)
    };

    // Get the asset func ID
    let asset_func_id = schema_variant.asset_func_id_or_error()?;

    Ok(LuminorkDefaultVariantMv::new(
        schema_id,
        default_variant_id,
        schema_variant.display_name().into(),
        schema_variant.category().into(),
        schema_variant.color().into(),
        schema_variant.is_locked(),
        schema_variant.description(),
        schema_variant.link(),
        asset_func_id,
        variant_func_ids,
        domain_props,
    ))
}
