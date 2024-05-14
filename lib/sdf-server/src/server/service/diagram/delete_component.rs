use axum::{extract::OriginalUri, http::uri::Uri};
use axum::{response::IntoResponse, Json};
use dal::diagram::SummaryDiagramComponent;
use dal::{ChangeSet, Component, ComponentId, DalContext, Visibility, WsEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

async fn delete_single_component(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<Option<Component>> {
    let comp = Component::get_by_id(ctx, component_id).await?;

    let id = comp.id();
    let comp_schema = comp.schema(ctx).await?;

    let maybe = comp.delete(ctx).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        "delete_component",
        serde_json::json!({
            "how": "/diagram/delete_component",
            "component_id": id,
            "component_schema_name": comp_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(maybe)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn delete_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteComponentsRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut components = HashMap::new();
    for component_id in request.component_ids {
        let maybe =
            delete_single_component(&ctx, component_id, &original_uri, &posthog_client).await?;
        components.insert(component_id, maybe.is_some());

        if let Some(maybe) = maybe {
            // to_delete=True
            let component: Component = Component::get_by_id(&ctx, maybe.id()).await?;
            let payload: SummaryDiagramComponent =
                SummaryDiagramComponent::assemble(&ctx, &component).await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        } // component_deleted called further down the stack
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&components)?)?)
}
