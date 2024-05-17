use axum::extract::OriginalUri;
use axum::Json;
use dal::change_set::ChangeSet;
use dal::Visibility;
use dal::{action::Action, DeprecatedAction};
use serde::{Deserialize, Serialize};

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    pub v2: bool,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn apply_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if request.v2 {
        // Removes v1 actions so they are not processed here
        for (_, bag) in DeprecatedAction::build_graph(&ctx).await? {
            bag.action.delete(&ctx).await?;
        }
    } else {
        // Removes v2 actions so they are not processed by the rebaser
        for action_id in Action::list_topologically(&ctx).await? {
            let action = Action::get_by_id(&ctx, action_id).await?;
            if action.is_eligible_to_dispatch() {
                Action::remove_by_id(&ctx, action_id).await?;
            }
        }
    }

    ctx.commit().await?;
    ctx.update_snapshot_to_visibility().await?;

    let change_set = ChangeSet::apply_to_base_change_set(&mut ctx, false).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": request.visibility.change_set_id,
        }),
    );

    // // If anything fails with uploading the workspace backup module, just log it. We shouldn't
    // // have the change set apply itself fail because of this.
    // tokio::task::spawn(
    //     super::upload_workspace_backup_module(ctx, raw_access_token)
    //         .instrument(info_span!("Workspace backup module upload")),
    // );

    ctx.commit().await?;

    Ok(Json(ApplyChangeSetResponse {
        change_set: change_set.to_owned(),
    }))
}
