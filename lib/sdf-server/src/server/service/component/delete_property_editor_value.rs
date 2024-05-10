use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::component::ComponentResult;
use axum::response::IntoResponse;
use axum::Json;
use dal::diagram::SummaryDiagramComponent;
use dal::{
    AttributeValue, AttributeValueId, ChangeSet, Component, ComponentId, PropId, Visibility,
    WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeletePropertyEditorValueRequest {
    pub attribute_value_id: AttributeValueId,
    pub prop_id: PropId,
    pub component_id: ComponentId,
    pub key: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn delete_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<DeletePropertyEditorValueRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    AttributeValue::remove_by_id(&ctx, request.attribute_value_id).await?;

    let component = Component::get_by_id(&ctx, request.component_id).await?;
    let payload: SummaryDiagramComponent =
        SummaryDiagramComponent::assemble(&ctx, &component).await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
