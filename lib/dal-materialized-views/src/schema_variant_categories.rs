use std::collections::{
    HashMap,
    HashSet,
};

use dal::{
    DalContext,
    SchemaId,
    SchemaVariant,
    cached_module::CachedModule,
};
use si_frontend_mv_types::{
    UninstalledVariant,
    schema_variant::{
        DisambiguateVariant,
        SchemaVariantCategories as SchemaVariantCategoriesMv,
        SchemaVariantsByCategory,
        VariantType,
    },
};
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.schema_variant_categories",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<SchemaVariantCategoriesMv> {
    let ctx = &ctx;
    let mut variant_by_category = HashMap::new();

    let installed = SchemaVariant::list_user_facing(ctx).await?;
    let mut installed_schema_ids = HashSet::new();
    let mut installed_cat_and_name = HashSet::new();
    for installed_variant in &installed {
        let category = installed_variant.category.as_str();
        let variants = variant_by_category
            .entry(category.to_owned())
            .or_insert(vec![]);
        variants.push(DisambiguateVariant {
            variant_type: VariantType::Installed,
            id: installed_variant.schema_variant_id.to_string(),
        });

        installed_schema_ids.insert(installed_variant.schema_id);
        installed_cat_and_name.insert((category, installed_variant.schema_name.as_str()));
    }

    let cached_modules: Vec<CachedModule> = CachedModule::latest_modules(ctx).await?;

    let mut uninstalled: HashMap<SchemaId, UninstalledVariant> = HashMap::new();
    // We want to hide uninstalled modules that would create duplicate assets in
    // the AssetPanel in old workspace. We do this just by name + category
    // matching. (We also hide if the schema is installed)
    for module in cached_modules {
        let category = module.category.to_owned();
        let category = category.as_deref().unwrap_or("");
        let schema_id = module.schema_id;
        let schema_name = module.schema_name.as_str();
        if !installed_schema_ids.contains(&module.schema_id)
            && !installed_cat_and_name.contains(&(category, schema_name))
        {
            let variants = variant_by_category
                .entry(category.to_owned())
                .or_insert(Vec::new());
            let uninstalled_variant = UninstalledVariant {
                schema_id: module.schema_id,
                schema_name: module.schema_name,
                display_name: module.display_name,
                category: module.category,
                link: module.link,
                color: module.color,
                description: module.description,
            };
            variants.push(DisambiguateVariant {
                variant_type: VariantType::Uninstalled,
                id: schema_id.to_string(), // is this right? no SV id? let's try schema id insted of module id
            });
            uninstalled.insert(schema_id, uninstalled_variant);
        }
    }

    let mut categories: Vec<SchemaVariantsByCategory> = vec![];
    for (name, variants) in variant_by_category.iter() {
        let mut variants = variants.to_vec();
        variants.sort_by_cached_key(|v| v.id.to_owned());
        categories.push(SchemaVariantsByCategory {
            display_name: name.to_string(),
            schema_variants: variants,
        });
    }
    categories.sort_by_cached_key(|c| c.display_name.to_owned());

    let workspace_mv_id = ctx.workspace_pk()?;
    Ok(SchemaVariantCategoriesMv {
        id: workspace_mv_id,
        categories,
        uninstalled,
    })
}
