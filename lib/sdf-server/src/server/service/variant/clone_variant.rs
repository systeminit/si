use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::{
    build_asset_func_spec, build_pkg_spec_for_variant, execute_asset_func, SchemaVariantError,
    SchemaVariantResult,
};
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::pkg::import_pkg_from_pkg;
use dal::schema::variant::SchemaVariantMetadataJson;
use dal::{
    generate_unique_id, ChangeSet, Func, SchemaVariant, SchemaVariantId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_pkg::SiPkg;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantRequest {
    pub id: SchemaVariantId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantResponse {
    pub id: SchemaVariantId,
    pub success: bool,
}

pub async fn clone_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CloneVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let variant_def = dbg!(SchemaVariant::get_by_id(&ctx, request.id).await)?;
    let schema = variant_def.schema(&ctx).await?;

    let new_name = format!("{} Clone {}", schema.name(), generate_unique_id(4));
    let menu_name = variant_def.display_name().map(|mn| format!("{mn} Clone"));

    if let Some(asset_func_id) = variant_def.asset_func_id() {
        let old_func = Func::get_by_id_or_error(&ctx, asset_func_id).await?;

        let cloned_func = old_func.duplicate(&ctx, new_name.clone()).await?;
        let cloned_func_spec = build_asset_func_spec(&cloned_func)?;
        let definition = execute_asset_func(&ctx, &cloned_func).await?;
        let metadata = SchemaVariantMetadataJson {
            name: new_name.clone(),
            menu_name: menu_name.clone(),
            category: variant_def.category().to_string(),
            color: variant_def.get_color(&ctx).await?,
            component_type: variant_def.component_type(),
            link: variant_def.link().clone(),
            description: variant_def.description().clone(),
        };

        //TODO @stack72 - figure out how we get the current user in this!
        let pkg_spec = build_pkg_spec_for_variant(
            definition,
            &cloned_func_spec,
            &metadata,
            "sally@systeminit.com",
        )?;

        let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

        let (_, schema_variant_ids, _) = import_pkg_from_pkg(
            &ctx,
            &pkg,
            Some(dal::pkg::ImportOptions {
                schemas: None,
                skip_import_funcs: Some(HashMap::from_iter([(
                    cloned_func_spec.unique_id.to_owned(),
                    cloned_func.clone(),
                )])),
                no_record: true,
                is_builtin: false,
            }),
        )
        .await?;

        let _schema_variant_id = schema_variant_ids
            .first()
            .copied()
            .ok_or(SchemaVariantError::NoAssetCreated)?;
    } else {
        return Err(SchemaVariantError::SchemaVariantAssetNotFound(request.id));
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "clone_variant",
        serde_json::json!({
            "variant_name": schema.name(),
            "variant_category": variant_def.category(),
            "variant_menu_name": variant_def.display_name(),
            "variant_id": variant_def.id(),
            "variant_component_type": variant_def.component_type(),
        }),
    );

    WsEvent::schema_variant_cloned(&ctx, variant_def.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&CloneVariantResponse {
        id: variant_def.id(),
        success: true,
    })?)?)
}
