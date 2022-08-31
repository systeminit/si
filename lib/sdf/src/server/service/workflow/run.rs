use axum::Json;
use serde::{Deserialize, Serialize};

use super::WorkflowResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{Visibility, WorkflowPrototypeId, WorkflowRunner};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunRequest {
    pub id: WorkflowPrototypeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunResponse {
    logs: Vec<String>,
}

pub async fn run(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<WorkflowRunRequest>,
) -> WorkflowResult<Json<WorkflowRunResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let (_, func_binding_return_values) = WorkflowRunner::run(&ctx, request.id).await?;
    let mut logs = Vec::new();
    for func_binding_return_value in func_binding_return_values {
        for stream in func_binding_return_value
            .get_output_stream(&ctx)
            .await?
            .unwrap_or_default()
        {
            match stream.data {
                Some(data) => logs.push(format!(
                    "{} {}",
                    stream.message,
                    serde_json::to_string_pretty(&data)?
                )),
                None => logs.push(stream.message),
            }
        }
    }

    txns.commit().await?;

    Ok(Json(WorkflowRunResponse { logs }))
}
