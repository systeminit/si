use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::{Json, OriginalUri};
use axum::response::IntoResponse;
use dal::{Action, ActionPrototypeId, ChangeSet, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddActionRequest {
    pub prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn add_action(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<AddActionRequest>,
) -> ChangeSetResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let action = Action::upsert(&ctx, request.prototype_id, request.component_id).await?;
    let prototype = action.prototype(&ctx).await?;
    let component = action.component(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_action",
        serde_json::json!({
            "how": "/change_set/add_action",
            "prototype_id": prototype.id,
            "prototype_kind": prototype.kind,
            "component_name": component.name(&ctx).await?,
            "component_id": component.id(),
            "change_set_pk": ctx.visibility().change_set_pk,
        }),
    );

    WsEvent::action_added(&ctx, component.id(), action.id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
