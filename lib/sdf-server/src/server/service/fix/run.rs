use axum::extract::OriginalUri;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::{FixError, FixResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use dal::job::definition::{FixItem, FixesJob};
use dal::{
    AttributeValueId, ComponentId, Fix, FixBatch, FixBatchId, HistoryActor, StandardModel, User,
    Visibility,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixRunRequest {
    pub attribute_value_id: AttributeValueId,
    pub component_id: ComponentId,
    pub action_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixesRunRequest {
    pub list: Vec<FixRunRequest>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixesRunResponse {
    pub id: FixBatchId,
}

pub async fn run(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<FixesRunRequest>,
) -> FixResult<Json<FixesRunResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
            .await?
            .ok_or(FixError::InvalidUser(*user_pk))?,

        HistoryActor::SystemInit => return Err(FixError::InvalidUserSystemInit),
    };
    let batch = FixBatch::new(&ctx, user.email()).await?;
    let mut fixes = Vec::with_capacity(request.list.len());
    for fix_run_request in request.list {
        let fix = Fix::new(
            &ctx,
            *batch.id(),
            fix_run_request.attribute_value_id,
            fix_run_request.component_id,
            &fix_run_request.action_name,
        )
        .await?;
        fixes.push(FixItem {
            id: *fix.id(),
            attribute_value_id: fix_run_request.attribute_value_id,
            component_id: fix_run_request.component_id,
            action: fix_run_request.action_name,
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
        .await;

    ctx.commit().await?;

    Ok(Json(FixesRunResponse { id: *batch.id() }))
}
