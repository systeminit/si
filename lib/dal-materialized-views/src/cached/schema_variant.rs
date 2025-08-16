use dal::{
    DalContext,
    SchemaVariantId,
    cached_module::CachedModule,
};
use si_frontend_mv_types::cached_schema_variant::CachedSchemaVariant as CachedSchemaVariantMv;
use si_id::FuncId;
use telemetry::prelude::*;

use super::collect_function_ids;

#[instrument(
    name = "dal_materialized_views.cached_schema_variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, id: SchemaVariantId) -> crate::Result<CachedSchemaVariantMv> {
    // Find the cached module containing this variant by storing the module info
    for mut module in CachedModule::latest_modules(&ctx).await? {
        let si_pkg = module.si_pkg(&ctx).await?;
        let schemas = si_pkg.schemas()?;
        for schema in schemas {
            let variants = schema.variants()?;
            for variant in variants {
                if let Some(unique_id) = variant.unique_id() {
                    if let Ok(variant_id) = unique_id.parse::<SchemaVariantId>() {
                        if variant_id == id {
                            // Found the variant, process it immediately to avoid lifetime issues
                            let variant_data = variant.data().ok_or_else(|| {
                                crate::Error::SchemaVariant(dal::SchemaVariantError::NotFound(id))
                            })?;

                            // Get the asset func unique_id and convert to FuncId
                            let asset_func_id = variant_data
                                .func_unique_id()
                                .parse::<FuncId>()
                                .map_err(|_| {
                                    crate::Error::SchemaVariant(dal::SchemaVariantError::NotFound(
                                        id,
                                    ))
                                })?;

                            // Get category from schema data
                            let category = schema
                                .data
                                .as_ref()
                                .map(|d| d.category())
                                .unwrap_or("Component") // default category
                                .to_string();

                            // Get display name from variant spec
                            let variant_spec = variant.to_spec().await?;
                            let display_name = variant_spec
                                .data
                                .as_ref()
                                .and_then(|d| d.display_name.as_ref())
                                .map(|d| d.to_string())
                                .unwrap_or_else(|| schema.name().to_string()); // fallback to schema name

                            // Collect all function IDs attached to this variant
                            let mut variant_func_ids = Vec::with_capacity(100); // Pre-allocate based on typical size
                            let schema_id_str = schema.unique_id().unwrap_or("unknown");
                            let variant_id_str = id.to_string();

                            // All function types use the same helper with closure syntax
                            collect_function_ids(|| variant.leaf_functions(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "leaf function")?;
                            collect_function_ids(|| variant.action_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "action function")?;
                            collect_function_ids(|| variant.auth_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "auth function")?;
                            collect_function_ids(|| variant.management_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "management function")?;
                            collect_function_ids(|| variant.si_prop_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "si prop function")?;
                            collect_function_ids(|| variant.root_prop_funcs(), &mut variant_func_ids, &schema_id_str, &variant_id_str, "root prop function")?;

                            // Remove duplicates efficiently, and ensure we have stable output for the MV.
                            variant_func_ids.sort_unstable();
                            variant_func_ids.dedup();

                            // Determine if this variant is the default variant for the schema
                            let is_default_variant = if let Some(schema_data) = schema.data.as_ref()
                            {
                                if let Some(default_variant_unique_id) =
                                    schema_data.default_schema_variant()
                                {
                                    // Check if this variant's unique_id matches the default
                                    variant.unique_id() == Some(default_variant_unique_id)
                                } else {
                                    // No default specified in schema, check if this is the first variant
                                    let all_variants = schema.variants()?;
                                    all_variants.first().map(|first| first.unique_id())
                                        == variant.unique_id().map(Some)
                                }
                            } else {
                                // No schema data, assume first variant is default
                                let all_variants = schema.variants()?;
                                all_variants.first().map(|first| first.unique_id())
                                    == variant.unique_id().map(Some)
                            };

                            return Ok(CachedSchemaVariantMv::new(
                                id,
                                display_name,
                                category,
                                variant_data.color().unwrap_or("").to_string(),
                                true, // is_locked - cached modules are locked until installed
                                variant_data.description().map(|d| d.to_string()),
                                variant_data.link().map(|l| l.to_string()),
                                asset_func_id,
                                variant_func_ids,
                                is_default_variant,
                            ));
                        }
                    }
                }
            }
        }
    }

    // Variant not found
    Err(crate::Error::SchemaVariant(
        dal::SchemaVariantError::NotFound(id),
    ))
}
