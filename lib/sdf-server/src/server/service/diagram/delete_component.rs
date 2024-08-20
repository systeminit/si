use axum::extract::Host;
use axum::{extract::OriginalUri, http::uri::Uri};
use axum::{response::IntoResponse, Json};
use dal::change_status::ChangeStatus;
use dal::component::frame::Frame;
use dal::diagram::SummaryDiagramComponent;
use dal::{ChangeSet, Component, ComponentId, DalContext, Visibility, WsEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    pub force_erase: bool,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn delete_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<DeleteComponentsRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut components = HashMap::new();
    let mut socket_map = HashMap::new();
    for component_id in request.component_ids {
        let component_still_exists = delete_single_component(
            &ctx,
            component_id,
            request.force_erase,
            &original_uri,
            &host_name,
            &posthog_client,
        )
        .await?;
        components.insert(component_id, component_still_exists);

        if component_still_exists {
            // to_delete=True
            let component: Component = Component::get_by_id(&ctx, component_id).await?;
            let payload: SummaryDiagramComponent = SummaryDiagramComponent::assemble(
                &ctx,
                &component,
                ChangeStatus::Deleted,
                &mut socket_map,
            )
            .await?;
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

async fn delete_single_component(
    ctx: &DalContext,
    component_id: ComponentId,
    force_erase: bool,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<bool> {
    let comp = Component::get_by_id(ctx, component_id).await?;

    let id = comp.id();
    let comp_schema = comp.schema(ctx).await?;

    let component_still_exists = if force_erase {
        if let Some(parent_id) = Component::get_parent_by_id(ctx, id).await? {
            for child_id in Component::get_children_for_id(ctx, id).await? {
                Frame::upsert_parent(ctx, child_id, parent_id).await?;
            }
        }

        Component::remove(ctx, id).await?;
        false
    } else {
        comp.delete(ctx).await?.is_some()
    };

    track(
        posthog_client,
        ctx,
        original_uri,
        host_name,
        "delete_component",
        serde_json::json!({
            "how": "/diagram/delete_component",
            "component_id": id,
            "component_schema_name": comp_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(component_still_exists)
}
