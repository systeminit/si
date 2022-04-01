use axum::extract::Query;
use axum::Json;
use dal::{Component, Schema, StandardModel, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{ApplicationError, ApplicationResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationItem {
    pub application: Component,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationResponse {
    pub list: Vec<ListApplicationItem>,
}

pub async fn list_applications(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListApplicationRequest>,
) -> ApplicationResult<Json<ListApplicationResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let schemas = Schema::find_by_attr(&ctx, "name", &"application".to_string()).await?;
    let schema = schemas.first().ok_or(ApplicationError::SchemaNotFound)?;
    let list: Vec<ListApplicationItem> = schema
        .components(&ctx)
        .await?
        .into_iter()
        .map(|application| ListApplicationItem { application })
        .collect();
    let response = ListApplicationResponse { list };

    txns.commit().await?;

    Ok(Json(response))
}
