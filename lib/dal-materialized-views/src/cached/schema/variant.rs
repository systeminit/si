use dal::{
    DalContext,
    SchemaVariantId,
    cached_module::CachedModule,
    schema::variant::DEFAULT_SCHEMA_VARIANT_COLOR,
};
use si_frontend_mv_types::cached_schema_variant::CachedSchemaVariant as CachedSchemaVariantMv;
use si_id::FuncId;
use si_pkg::{
    SiPkgSchema,
    SiPkgSchemaVariant,
};
use telemetry::prelude::*;

use crate::cached::collect_function_ids;

/// Data structure for assembled variant information
pub(crate) struct AssembledVariantData {
    pub variant_id: SchemaVariantId,
    pub display_name: String,
    pub category: String,
    pub color: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub asset_func_id: FuncId,
    pub variant_func_ids: Vec<FuncId>,
}

/// Assembles variant data from schema and variant components
pub(crate) async fn assemble_variant_data(
    schema: &SiPkgSchema<'_>,
    variant: &SiPkgSchemaVariant<'_>,
    variant_id: SchemaVariantId,
) -> crate::Result<AssembledVariantData> {
    // Get variant data
    let variant_data = variant.data().ok_or_else(|| {
        crate::Error::SchemaVariant(dal::SchemaVariantError::NotFound(variant_id))
    })?;

    // Get the asset func unique_id and convert to FuncId
    let asset_func_id = variant_data
        .func_unique_id()
        .parse::<FuncId>()
        .map_err(|_| crate::Error::SchemaVariant(dal::SchemaVariantError::NotFound(variant_id)))?;

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
    let variant_id_str = variant_id.to_string();

    // All function types use the same helper with closure syntax
    collect_function_ids(
        || variant.leaf_functions(),
        &mut variant_func_ids,
        schema_id_str,
        &variant_id_str,
        "leaf function",
    )?;
    collect_function_ids(
        || variant.action_funcs(),
        &mut variant_func_ids,
        schema_id_str,
        &variant_id_str,
        "action function",
    )?;
    collect_function_ids(
        || variant.auth_funcs(),
        &mut variant_func_ids,
        schema_id_str,
        &variant_id_str,
        "auth function",
    )?;
    collect_function_ids(
        || variant.management_funcs(),
        &mut variant_func_ids,
        schema_id_str,
        &variant_id_str,
        "management function",
    )?;
    collect_function_ids(
        || variant.si_prop_funcs(),
        &mut variant_func_ids,
        schema_id_str,
        &variant_id_str,
        "si prop function",
    )?;
    collect_function_ids(
        || variant.root_prop_funcs(),
        &mut variant_func_ids,
        schema_id_str,
        &variant_id_str,
        "root prop function",
    )?;

    // Remove duplicates efficiently, and ensure we have stable output for the MV.
    variant_func_ids.sort_unstable();
    variant_func_ids.dedup();

    Ok(AssembledVariantData {
        variant_id,
        display_name,
        category,
        color: variant_data
            .color()
            .unwrap_or(DEFAULT_SCHEMA_VARIANT_COLOR)
            .to_string(),
        description: variant_data.description().map(|d| d.to_string()),
        link: variant_data.link().map(|l| l.to_string()),
        asset_func_id,
        variant_func_ids,
    })
}

#[instrument(
    name = "dal_materialized_views.cached_schema_variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    id: SchemaVariantId,
) -> crate::Result<CachedSchemaVariantMv> {
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
                            // Found the variant, assemble its data
                            let assembled_data =
                                assemble_variant_data(&schema, &variant, id).await?;

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
                                assembled_data.variant_id,
                                assembled_data.display_name,
                                assembled_data.category,
                                assembled_data.color,
                                true, // is_locked - cached modules are locked until installed
                                assembled_data.description,
                                assembled_data.link,
                                assembled_data.asset_func_id,
                                assembled_data.variant_func_ids,
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

pub mod default;
