use super::SchemaVariantDefinitionResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    component::ComponentKind,
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantDefRequest {
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub component_kind: Option<ComponentKind>,
    pub link: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantDefResponse {
    pub id: SchemaVariantDefinitionId,
    pub success: bool,
}

pub async fn create_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<CreateVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::new(
        &ctx,
        request.name,
        request.menu_name,
        request.category,
        request.link,
        request.color,
        request.component_kind.unwrap_or(ComponentKind::Standard),
        "".to_string(),
    )
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(CreateVariantDefResponse {
        id: *variant_def.id(),
        success: true,
    }))
}
