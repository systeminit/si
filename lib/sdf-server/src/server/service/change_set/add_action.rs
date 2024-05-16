use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::{Json, OriginalUri};
use axum::response::IntoResponse;
use dal::action::prototype::ActionKind;
use dal::{
    action::prototype::ActionPrototype, action::Action, ActionPrototypeId, ChangeSet, Component,
    ComponentId, DeprecatedAction, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddActionRequest {
    pub prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    pub v2: bool,
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

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    if request.v2 {
        let prototype = ActionPrototype::get_by_id(&ctx, request.prototype_id).await?;

        match prototype.kind {
            ActionKind::Create | ActionKind::Destroy | ActionKind::Update | ActionKind::Refresh => {
                let maybe_duplicate_action = Action::find_for_kind_and_component_id(
                    &ctx,
                    request.component_id,
                    prototype.kind,
                )
                .await?;
                if !maybe_duplicate_action.is_empty() {
                    return Err(super::ChangeSetError::ActionAlreadyEnqueued(prototype.id));
                }
            }

            dal::action::prototype::ActionKind::Manual => {}
        }

        let component = Component::get_by_id(&ctx, request.component_id).await?;
        let action = Action::new(&ctx, request.prototype_id, Some(request.component_id)).await?;
        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "create_action_v2",
            serde_json::json!({
                "how": "/change_set/add_action",
                "action_id": action.id(),
                "action_kind": prototype.kind,
                "component_id": component.id(),
                "change_set_id": ctx.change_set_id(),
            }),
        );
        // todo add ws event here
    } else {
        let action =
            DeprecatedAction::upsert(&ctx, request.prototype_id, request.component_id).await?;
        let prototype = action.prototype(&ctx).await?;
        let component = action.component(&ctx).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "create_action_v1",
            serde_json::json!({
                "how": "/change_set/add_action",
                "action_id": action.id.clone(),
                "action_kind": prototype.kind,
                "component_id": component.id(),
                "change_set_id": ctx.change_set_id(),
            }),
        );
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
