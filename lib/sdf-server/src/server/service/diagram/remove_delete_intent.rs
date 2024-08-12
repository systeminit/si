use axum::extract::Host;
use axum::Json;
use axum::{extract::OriginalUri, http::uri::Uri, response::IntoResponse};
use dal::{ChangeSet, Component, ComponentId, DalContext, Visibility};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

async fn remove_single_delete_intent(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {
    let comp = Component::get_by_id(ctx, component_id).await?;

    let comp_schema = comp.schema(ctx).await?;
    let comp = comp.set_to_delete(ctx, false).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        host_name,
        "remove_delete_intent",
        serde_json::json!({
            "how": "/diagram/remove_delete_intent",
            "component_id": comp.id(),
            "component_schema_name": comp_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(())
}

async fn restore_component_from_base_change_set(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {
    Component::restore_from_base_change_set(ctx, component_id).await?;
    let comp = Component::get_by_id(ctx, component_id).await?;
    let comp_schema = comp.schema(ctx).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        host_name,
        "restore_from_base_change_set",
        serde_json::json!({
            "how": "/diagram/remove_delete_intent",
            "component_id": component_id,
            "component_schema_name": comp_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDeleteIntentComponentInformation {
    pub component_id: ComponentId,
    pub from_base_change_set: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDeleteIntentRequest {
    pub components: Vec<RemoveDeleteIntentComponentInformation>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Restore a set of [`Component`](dal::Component)s via their componentId. Creating change set if on head.
pub async fn remove_delete_intent(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<RemoveDeleteIntentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    for component_info in request.components {
        match component_info.from_base_change_set {
            true => {
                restore_component_from_base_change_set(
                    &ctx,
                    component_info.component_id,
                    &original_uri,
                    &host_name,
                    &posthog_client,
                )
                .await?
            }
            false => {
                remove_single_delete_intent(
                    &ctx,
                    component_info.component_id,
                    &original_uri,
                    &host_name,
                    &posthog_client,
                )
                .await?
            }
        }
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
