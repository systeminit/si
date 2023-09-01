use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::service::change_set::ChangeSetError;
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::job::definition::{FixItem, FixesJob};
use dal::{
    AttributeValueId, ChangeSet, ChangeSetPk, Fix, FixBatch, HistoryActor, StandardModel, User,
};
use serde::{Deserialize, Serialize};
//use telemetry::tracing::{info_span, Instrument, log::warn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    pub change_set_pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn apply_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let mut ctx = builder.build_head(access_builder).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    let actions = change_set.actions(&ctx).await?;
    change_set.apply_raw(&mut ctx, false).await?;

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

    if !actions.is_empty() {
        let batch = FixBatch::new(&ctx, user.email()).await?;
        let mut fixes = Vec::with_capacity(actions.len());

        for action in actions {
            let fix = Fix::new(
                &ctx,
                *batch.id(),
                // TODO: remove this
                AttributeValueId::NONE,
                *action.component_id(),
                *action.action_prototype_id(),
            )
            .await?;

            fixes.push(FixItem {
                id: *fix.id(),
                attribute_value_id: AttributeValueId::NONE,
                component_id: *action.component_id(),
                action_prototype_id: *action.action_prototype_id(),
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

    // If anything fails with uploading the workspace backup module, just log it. We shouldn't
    // have the change set apply itself fail because of this.
    /*
    tokio::task::spawn(
        super::upload_workspace_backup_module(ctx, raw_access_token)
            .instrument(info_span!("Workspace backup module upload")),
    );
    */

    Ok(Json(ApplyChangeSetResponse { change_set }))
}
