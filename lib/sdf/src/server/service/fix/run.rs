use axum::Json;
use serde::{Deserialize, Serialize};

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::job::definition::{FixItem, FixesJob};
use dal::{ComponentId, ConfirmationResolverId, Fix, FixBatch, StandardModel, Visibility};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixRunRequest {
    pub id: ConfirmationResolverId,
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
    success: bool,
}

pub async fn run(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<FixesRunRequest>,
) -> FixResult<Json<FixesRunResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // TODO(nick): use the logged-in user account information.
    let batch = FixBatch::new(&ctx, "toddhoward@systeminit.com").await?;
    let mut fixes = Vec::with_capacity(request.list.len());
    for fix_run_request in request.list {
        let fix = Fix::new(
            &ctx,
            *batch.id(),
            fix_run_request.id,
            fix_run_request.component_id,
        )
        .await?;
        fixes.push(FixItem {
            id: *fix.id(),
            confirmation_resolver_id: fix_run_request.id,
            component_id: fix_run_request.component_id,
            action: fix_run_request.action_name,
        });
    }

    ctx.enqueue_job(FixesJob::new(&ctx, fixes, *batch.id()))
        .await;

    ctx.commit().await?;

    Ok(Json(FixesRunResponse { success: true }))
}
