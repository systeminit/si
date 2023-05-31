use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::service::change_set::ChangeSetError;
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::job::definition::{FixItem, FixesJob};
use dal::{
    ActionPrototypeId, AttributeValueId, ChangeSet, ChangeSetPk, ComponentId, Fix, FixBatch,
    HistoryActor, StandardModel, User,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixRunRequest {
    pub attribute_value_id: AttributeValueId,
    pub component_id: ComponentId,
    pub action_prototype_id: ActionPrototypeId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    pub change_set_pk: ChangeSetPk,
    pub list: Vec<FixRunRequest>,
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
    let mut ctx = builder.build(request_ctx.build_head()).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.apply(&mut ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": request.change_set_pk,
        }),
    );

    ctx.blocking_commit().await?;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
            .await?
            .ok_or(ChangeSetError::InvalidUser(*user_pk))?,

        HistoryActor::SystemInit => return Err(ChangeSetError::InvalidUserSystemInit),
    };
    if !request.list.is_empty() {
        let batch = FixBatch::new(&ctx, user.email()).await?;
        let mut fixes = Vec::with_capacity(request.list.len());

        for fix_run_request in request.list {
            let fix = Fix::new(
                &ctx,
                *batch.id(),
                fix_run_request.attribute_value_id,
                fix_run_request.component_id,
                fix_run_request.action_prototype_id,
            )
            .await?;

            fixes.push(FixItem {
                id: *fix.id(),
                attribute_value_id: fix_run_request.attribute_value_id,
                component_id: fix_run_request.component_id,
                action_prototype_id: fix_run_request.action_prototype_id,
            });
        }

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "apply_fix",
            serde_json::json!({
                "fix_batch_id": batch.id(),
                "number_of_fixes_in_batch": fixes.len(),
                "fixes_applied": fixes,
            }),
        );

        ctx.enqueue_job(FixesJob::new(&ctx, fixes, *batch.id()))
            .await?;
    }

    ctx.commit().await?;

    Ok(Json(ApplyChangeSetResponse { change_set }))
}
