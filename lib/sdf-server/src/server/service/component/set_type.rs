use axum::{response::IntoResponse, Json};

use dal::{ChangeSet, Component, ComponentId, ComponentType, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetTypeRequest {
    pub component_id: ComponentId,
    pub value: Option<serde_json::Value>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn set_type(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetTypeRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let component = Component::get_by_id(&ctx, request.component_id).await?;

    // If no type was found, default to a standard "component".
    let component_type: ComponentType = match request.value {
        Some(value) => serde_json::from_value(value)?,
        None => ComponentType::Component,
    };
    component.set_type(&ctx, component_type).await?;

    // TODO(Wendy) - uncomment this when we restore posthog tracking
    // TODO(Wendy) - replace this old component_schema code
    // let component_schema = component
    //     .schema(&ctx)
    //     .await?
    //     .ok_or(ComponentError::SchemaNotFound)?;
    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "set_component_type",
    //     serde_json::json!({
    //                 "component_id": component.id(),
    //                 "component_schema_name": component_schema.name(),
    //                 "new_component_type": component_type,
    //     }),
    // );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
