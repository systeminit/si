use axum::{extract::Query, Json};
use dal::{ActionKind, ActionPrototype, ActionPrototypeView, Component, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetActionsResponse {
    pub actions: Vec<ActionPrototypeView>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetActionsRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn get_actions(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetActionsRequest>,
) -> ComponentResult<Json<GetActionsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema_variant = Component::get_by_id(&ctx, request.component_id)
        .await?
        .schema_variant(&ctx)
        .await?;

    let action_prototypes = ActionPrototype::for_variant(&ctx, schema_variant.id()).await?;
    let mut action_views: Vec<ActionPrototypeView> = Vec::new();
    for action_prototype in action_prototypes {
        if action_prototype.kind == ActionKind::Refresh {
            continue;
        }

        let view = ActionPrototypeView::new(&ctx, action_prototype).await?;
        action_views.push(view);
    }

    Ok(Json(GetActionsResponse {
        actions: action_views,
    }))
}
