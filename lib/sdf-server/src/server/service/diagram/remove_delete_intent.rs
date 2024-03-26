use axum::Json;
use axum::{extract::OriginalUri, http::uri::Uri, response::IntoResponse};
use dal::{ChangeSetPointer, Component, ComponentId, DalContext, Visibility};
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
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {

    let comp = Component::get_by_id(ctx, component_id).await?;

    let comp_schema = comp.schema(ctx).await?;
    let comp = comp.set_to_delete(ctx, false).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        "restore_component",
        serde_json::json!({
                    "component_id": comp.id(),
                    "component_schema_name": comp_schema.name(),
        }),
    );

    Ok(())
}


#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDeleteIntentRequest {
    pub component_ids: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Restore a set of [`Component`](dal::Component)s via their componentId. Creating change set if on head.
pub async fn remove_delete_intent(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RemoveDeleteIntentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSetPointer::force_new(&mut ctx).await?;

    for component_id in request.component_ids {
        remove_single_delete_intent(&ctx, component_id, &original_uri, &posthog_client).await?;
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
