use axum::{response::IntoResponse, Json};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::AttributePrototypeArgumentBag;
use dal::{
    ChangeSet, ComponentId, FuncId, OutputSocketId, PropId, SchemaVariantId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttributePrototypeRequest {
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
    component_id: Option<ComponentId>,
    prop_id: Option<PropId>,
    output_socket_id: Option<OutputSocketId>,
    prototype_arguments: Vec<AttributePrototypeArgumentBag>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn create_attribute_prototype(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateAttributePrototypeRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::create_attribute_prototype(
        &ctx,
        request.func_id,
        request.schema_variant_id,
        request.component_id,
        request.prop_id,
        request.output_socket_id,
        request.prototype_arguments,
    )
    .await?;

    WsEvent::func_saved(&ctx, request.func_id)
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
