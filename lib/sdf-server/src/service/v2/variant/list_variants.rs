use std::collections::HashSet;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSetId,
    SchemaVariant,
    WorkspacePk,
    cached_module::CachedModule,
};
use si_frontend_types::{
    ListVariantsResponse,
    UninstalledVariant,
};

use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::{
        AccessBuilder,
        variant::SchemaVariantsAPIError,
    },
    track,
};

pub async fn list_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<ListVariantsResponse>, SchemaVariantsAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let installed = SchemaVariant::list_user_facing(&ctx).await?;

    let mut installed_schema_ids = HashSet::new();
    let mut installed_cat_and_name = HashSet::new();
    for installed_variant in &installed {
        installed_schema_ids.insert(installed_variant.schema_id);
        installed_cat_and_name.insert((
            installed_variant.category.as_str(),
            installed_variant.schema_name.as_str(),
        ));
    }

    let cached_modules: Vec<CachedModule> = CachedModule::latest_modules(&ctx).await?;

    let mut uninstalled = vec![];
    // We want to hide uninstalled modules that would create duplicate assets in
    // the AssetPanel in old workspace. We do this just by name + category
    // matching. (We also hide if the schema is installed)
    for module in cached_modules {
        let category = module.category.as_deref().unwrap_or("");

        let schema_name = module.schema_name.as_str();
        if !installed_schema_ids.contains(&module.schema_id)
            && !installed_cat_and_name.contains(&(category, schema_name))
        {
            uninstalled.push(UninstalledVariant {
                schema_id: module.schema_id,
                schema_name: module.schema_name,
                display_name: module.display_name,
                category: module.category,
                link: module.link,
                color: module.color,
                description: module.description,
                component_type: module.component_type.into(),
            });
        }
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "list_variants",
        serde_json::json!({}),
    );

    Ok(Json(ListVariantsResponse {
        installed,
        uninstalled,
    }))
}
