use axum::{extract::Query, Json};
use dal::diagram::Diagram;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagramRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetDiagramResponse = Diagram;

pub async fn get_diagram(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiagramRequest>,
) -> DiagramResult<Json<GetDiagramResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    // let response = Diagram::assemble(&ctx).await?;
    {
        let mut s = ctx.workspace_snapshot().unwrap().try_lock().unwrap();
        s.tiny_dot_to_file();
    }
    let response = Diagram {
        components: vec![],
        edges: vec![],
    };
    Ok(Json(response))
}
