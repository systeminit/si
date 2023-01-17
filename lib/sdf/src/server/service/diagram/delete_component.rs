use axum::Json;
use dal::{Component, ComponentId, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a [`Component`](dal::Component) via its componentId.
pub async fn delete_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<DeleteComponentRequest>,
) -> DiagramResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // get the status of protection
    // if it's protected then throw a DiagramError
    // else do nothing (for now)
    let comp = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let protection_opt = comp
        .find_value_by_json_pointer::<bool>(&ctx, "/root/si/protected")
        .await?;

    if let Some(protection) = protection_opt {
        if protection {
            return Err(DiagramError::ComponentProtected(request.component_id));
        }
    }

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}
