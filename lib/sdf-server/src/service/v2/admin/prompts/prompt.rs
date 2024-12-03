use asset_sprayer::prompt::AwsCliCommandPromptKind;
use axum::{
    extract::Path,
    routing::{delete, get, put},
    Json, Router,
};
use dal::prompt_override::PromptOverride;
use serde::{Deserialize, Serialize};

use crate::{
    extract::{AccessBuilder, AssetSprayer, HandlerContext},
    AppState,
};

use super::Result;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_prompt))
        .route("/", put(set_prompt))
        .route("/", delete(reset_prompt))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptValue {
    pub kind: AwsCliCommandPromptKind,
    pub prompt_yaml: String,
    pub overridden: bool,
}

pub async fn get_prompt(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    AssetSprayer(asset_sprayer): AssetSprayer,
    Path(kind): Path<AwsCliCommandPromptKind>,
) -> Result<Json<PromptValue>> {
    let ctx = builder.build_head(access_builder).await?;
    let prompt_override = PromptOverride::get_opt(&ctx, kind.as_ref()).await?;
    let overridden = prompt_override.is_some();
    let prompt_yaml = match prompt_override {
        Some(prompt_override) => prompt_override,
        None => asset_sprayer.raw_prompt(kind).await?.to_string(),
    };
    Ok(Json(PromptValue {
        kind,
        prompt_yaml,
        overridden,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPromptRequest {
    pub prompt_yaml: String,
}

pub async fn set_prompt(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path(kind): Path<AwsCliCommandPromptKind>,
    Json(request): Json<SetPromptRequest>,
) -> Result<()> {
    let ctx = builder.build_head(access_builder).await?;
    PromptOverride::set(&ctx, kind.as_ref(), &request.prompt_yaml).await?;
    ctx.commit_no_rebase().await?;
    Ok(())
}

pub async fn reset_prompt(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path(kind): Path<AwsCliCommandPromptKind>,
) -> Result<()> {
    let ctx = builder.build_head(access_builder).await?;
    PromptOverride::reset(&ctx, kind.as_ref()).await?;
    ctx.commit_no_rebase().await?;
    Ok(())
}
