use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    StandardModel, Visibility, WsEvent,
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
    pub definition: String,
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
    Json(request): Json<SaveVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<SaveVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    variant_def.set_name(&ctx, request.name).await?;
    variant_def.set_menu_name(&ctx, request.menu_name).await?;
    variant_def.set_category(&ctx, request.category).await?;
    variant_def.set_color(&ctx, request.color).await?;
    variant_def.set_link(&ctx, request.link).await?;
    variant_def
        .set_description(&ctx, request.description)
        .await?;
    variant_def.set_definition(&ctx, request.definition).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(SaveVariantDefResponse { success: true }))
}
