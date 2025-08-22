use dal::{
    DalContext,
    SchemaId,
    SchemaVariantId,
    cached_module::CachedModule,
};
use si_frontend_mv_types::cached_schema::CachedSchema as CachedSchemaMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.cached_schema",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, id: SchemaId) -> crate::Result<CachedSchemaMv> {
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

    // Find the default variant based on the schema data
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
    let default_variant_id: SchemaVariantId = default_variant
        .unique_id()
        .ok_or_else(|| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?
        .parse::<SchemaVariantId>()
        .map_err(|_| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    // Collect all variant IDs from their unique_ids
    let mut variant_ids = Vec::new();
    for variant in &variants {
        if let Some(unique_id) = variant.unique_id() {
            if let Ok(variant_id) = unique_id.parse::<SchemaVariantId>() {
                variant_ids.push(variant_id);
            }
        }
    }

    Ok(CachedSchemaMv::new(
        id,
        module.schema_name,
        default_variant_id,
        variant_ids,
    ))
}

pub mod prop_conversion;
pub mod variant;
