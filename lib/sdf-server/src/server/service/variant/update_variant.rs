use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, Func, SchemaVariant, SchemaVariantId, WsEvent};
use dal::{ComponentType, SchemaId, Visibility};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantRequest {
    pub id: SchemaId,
    pub default_schema_variant_id: SchemaVariantId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub code: String,
    pub description: Option<String>,
    pub component_type: ComponentType,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantResponse {
    pub success: bool,
}

pub async fn update_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExecVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let sv = SchemaVariant::get_by_id(&ctx, request.default_schema_variant_id).await?;
    let components_in_use = sv.get_components_on_graph(&ctx).await?;

    let updated_sv_id: SchemaVariantId = if let Some(asset_func_id) = sv.asset_func_id() {
        let asset_func = Func::get_by_id_or_error(&ctx, asset_func_id).await?;
        if !components_in_use.is_empty() {
            // If we have components_in_use
            // We should create a new version of the schema_variant
            // and we will set that new version to be the default for the schema
            VariantAuthoringClient::update_and_generate_variant_with_new_version(
                &ctx,
                &asset_func,
                sv.id(),
                request.name.clone(),
                request.menu_name.clone(),
                request.category.clone(),
                request.color.clone(),
                request.link.clone(),
                request.code.clone(),
                request.description.clone(),
                request.component_type,
            )
            .await?
        } else {
            VariantAuthoringClient::update_existing_variant_and_regenerate(
                &ctx,
                sv.id(),
                request.name.clone(),
                request.menu_name.clone(),
                request.category.clone(),
                request.color.clone(),
                request.link.clone(),
                request.code.clone(),
                request.description.clone(),
                request.component_type,
            )
            .await?;
            sv.id()
        }
    } else {
        return Err(
            crate::service::variant::SchemaVariantError::SchemaVariantAssetNotFound(sv.id()),
        );
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "update_variant",
        serde_json::json!({
            "variant_name": request.name.clone(),
            "variant_category": request.category.clone(),
            "variant_menu_name": request.menu_name.clone(),
            "variant_id": updated_sv_id.clone(),
        }),
    );

    WsEvent::schema_variant_update_finished(&ctx, updated_sv_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&ExecVariantResponse {
        success: true,
    })?)?)
}
