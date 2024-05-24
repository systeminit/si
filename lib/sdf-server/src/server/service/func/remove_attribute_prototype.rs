use axum::{response::IntoResponse, Json};
use dal::func::authoring::FuncAuthoringClient;
use dal::{AttributePrototype, AttributePrototypeId, ChangeSet, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveAttributePrototypeRequest {
    attribute_prototype_id: AttributePrototypeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn remove_attribute_prototype(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RemoveAttributePrototypeRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func_id = AttributePrototype::func_id(&ctx, request.attribute_prototype_id).await?;
    FuncAuthoringClient::remove_attribute_prototype(&ctx, request.attribute_prototype_id).await?;

    WsEvent::func_saved(&ctx, func_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
