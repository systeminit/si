use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    Func, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub code: String,
    pub handler: String,
    pub description: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveVariantDefResponse {
    pub success: bool,
}

pub async fn save_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SaveVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<SaveVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;
    variant_def.set_name(&ctx, request.name.clone()).await?;
    variant_def
        .set_menu_name(&ctx, request.menu_name.clone())
        .await?;
    variant_def
        .set_category(&ctx, request.category.clone())
        .await?;
    variant_def.set_color(&ctx, request.color).await?;
    variant_def.set_link(&ctx, request.link).await?;
    variant_def
        .set_description(&ctx, request.description)
        .await?;

    let mut asset_func = Func::get_by_id(&ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;
    asset_func
        .set_code_plaintext(&ctx, Some(&request.code))
        .await?;
    asset_func.set_handler(&ctx, Some(request.handler)).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_variant_def",
        serde_json::json!({
                    "variant_def_category": request.category,
                    "variant_def_name": request.name,
                    "variant_def_menu_name": request.menu_name,
                    // "variant_def_definition":  request.definition,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(SaveVariantDefResponse { success: true }))
}
