use std::collections::HashMap;

use axum::Json;
use dal::{
    AttributeValue, AttributeValueId, ChangeSet, Component, ComponentId, PropId, Visibility,
    WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{AccessBuilder, HandlerContext},
    service::{component::ComponentResult, force_change_set_response::ForceChangeSetResponse},
};

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
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    AttributeValue::remove_by_id(&ctx, request.attribute_value_id).await?;

    let component = Component::get_by_id(&ctx, request.component_id).await?;
    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(&ctx, component.change_status(&ctx).await?, &mut socket_map)
        .await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
