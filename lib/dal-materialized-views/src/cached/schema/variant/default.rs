use dal::{
    DalContext,
    SchemaId,
    cached_module::CachedModule,
};
use si_frontend_mv_types::cached_default_variant::CachedDefaultVariant as CachedDefaultVariantMv;
use telemetry::prelude::*;

use super::assemble_variant_data;

#[instrument(
    name = "dal_materialized_views.cached_default_variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, id: SchemaId) -> crate::Result<CachedDefaultVariantMv> {
    let mut module = CachedModule::find_latest_for_schema_id(&ctx, id)
        .await?
        .ok_or_else(|| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    // Get the SiPkg data to extract variant information
    let si_pkg = module.si_pkg(&ctx).await?;
    let schemas = si_pkg.schemas()?;
    let schema = schemas
        .into_iter()
        .next()
        .ok_or_else(|| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    let variants = schema.variants()?;

    // === Find the default variant based on the schema data ===
    let schema_data = schema
        .data()
        .ok_or_else(|| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    let default_variant =
        if let Some(default_variant_unique_id) = schema_data.default_schema_variant() {
            // Find the variant with the matching unique_id
            variants
                .iter()
                .find(|v| v.unique_id() == Some(default_variant_unique_id))
                .or_else(|| variants.first()) // Fallback to first variant
        } else {
            // No default specified, use first variant
            variants.first()
        }
        .ok_or_else(|| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    // Extract the variant unique_id and convert to SchemaVariantId
    let default_variant_id: dal::SchemaVariantId = default_variant
        .unique_id()
        .ok_or_else(|| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?
        .parse()
        .map_err(|_| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    // Assemble the variant data
    let assembled_data =
        assemble_variant_data(&schema, default_variant, default_variant_id).await?;

    Ok(CachedDefaultVariantMv::new(
        id, // schema_id as the MV object ID
        assembled_data.variant_id,
        assembled_data.display_name,
        assembled_data.category,
        assembled_data.color,
        true, // All cached schema variants are locked by definition
        assembled_data.description,
        assembled_data.link,
        assembled_data.asset_func_id,
        assembled_data.variant_func_ids,
        assembled_data.domain_props,
    ))
}
