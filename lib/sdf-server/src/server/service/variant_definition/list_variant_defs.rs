use super::SchemaVariantDefinitionResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::{
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    StandardModel, Timestamp, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantDefsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListedVariantDef {
    pub id: SchemaVariantDefinitionId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantDefsResponse {
    pub variant_defs: Vec<ListedVariantDef>,
}

pub async fn list_variant_defs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<ListVariantDefsRequest>,
) -> SchemaVariantDefinitionResult<Json<ListVariantDefsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_defs: Vec<ListedVariantDef> =
        SchemaVariantDefinition::list_for_default_variants(&ctx)
            .await?
            .iter()
            .map(|def| ListedVariantDef {
                // TODO: Ensure we pass an actor for created / updated / deleted to the frontend
                id: def.id().to_owned(),
                name: def.name().to_owned(),
                menu_name: def.menu_name().map(|menu_name| menu_name.to_owned()),
                category: def.category().to_owned(),
                color: def.color().to_owned(),
                timestamp: def.timestamp().to_owned(),
            })
            .collect();

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_variant_def",
        serde_json::json!({}),
    );

    Ok(Json(ListVariantDefsResponse { variant_defs }))
}
