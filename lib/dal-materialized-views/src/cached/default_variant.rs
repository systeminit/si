use dal::{
    DalContext,
    SchemaId,
    cached_module::CachedModule,
};
use si_frontend_mv_types::cached_default_variant::CachedDefaultVariant as CachedDefaultVariantMv;
use si_id::FuncId;
use telemetry::prelude::*;

use super::collect_function_ids;

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

    // Get default variant data
    let default_variant_data = default_variant
        .data()
        .ok_or_else(|| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    // === Collect all function IDs for the default variant ===
    let mut variant_func_ids = Vec::new();
    let schema_id_str = id.to_string();
    let variant_id_str = default_variant_id.to_string();

    // Leaf funcs
    collect_function_ids(|| default_variant.leaf_functions(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "leaf function")?;

    // Action funcs
    collect_function_ids(|| default_variant.action_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "action function")?;

    // Auth funcs
    collect_function_ids(|| default_variant.auth_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "auth function")?;

    // Management funcs
    collect_function_ids(|| default_variant.management_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "management function")?;

    // SiProp funcs
    collect_function_ids(|| default_variant.si_prop_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "si prop function")?;

    // RootProp funcs
    collect_function_ids(|| default_variant.root_prop_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "root prop function")?;

    // Get asset func ID from the variant data
    let asset_func_id = default_variant_data
        .func_unique_id()
        .parse::<FuncId>()
        .map_err(|_| crate::Error::Schema(dal::SchemaError::UninstalledSchemaNotFound(id)))?;

    // === Build the response with sensible defaults ===
    // Get category from schema data
    let category = schema
        .data
        .as_ref()
        .map(|d| d.category())
        .unwrap_or("Component") // default category
        .to_string();

    // Get display name from variant spec
    let variant_spec = default_variant.to_spec().await?;
    let display_name = variant_spec
        .data
        .as_ref()
        .and_then(|d| d.display_name.as_ref())
        .map(|d| d.to_string())
        .unwrap_or_else(|| format!("Unnamed Variant {}", default_variant_id));

    Ok(CachedDefaultVariantMv::new(
        id, // schema_id as the MV object ID
        default_variant_id,
        display_name,
        category,
        default_variant_data.color().unwrap_or("#000000").to_string(),
        true, // All cached schema variants are locked by definition
        default_variant_data.description().map(|s| s.to_string()),
        default_variant_data.link().map(|s| s.to_string()),
        asset_func_id,
        variant_func_ids,
    ))
}
