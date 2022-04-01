use axum::extract::Query;
use axum::Json;
use dal::system::UNSET_SYSTEM_ID;
use dal::{CodeLanguage, CodeView, Component, ComponentId, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeResponse {
    pub code_views: Vec<CodeView>,
}

pub async fn get_code(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetCodeRequest>,
) -> ComponentResult<Json<GetCodeResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let system_id = request.system_id.unwrap_or(UNSET_SYSTEM_ID);
    let code_list =
        Component::list_code_generated_by_component_id(&ctx, request.component_id, system_id)
            .await?;

    let code_views: Vec<CodeView> = code_list
        .into_iter()
        .map(|code_gen| CodeView::new(CodeLanguage::Yaml, code_gen.code))
        .collect();

    txns.commit().await?;
    Ok(Json(GetCodeResponse { code_views }))
}
