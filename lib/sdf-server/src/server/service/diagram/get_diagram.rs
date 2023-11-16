use axum::{extract::Query, Json};
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

// FIXME(nick): this is a fake diagram! Replace this!
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagram {
    components: Vec<()>,
    edges: Vec<()>,
}

pub type GetDiagramResponse = Diagram;

pub async fn get_diagram(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiagramRequest>,
) -> DiagramResult<Json<GetDiagramResponse>> {
    let _ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // FIXME(nick): use a real response.
    // let response = Diagram::assemble(&ctx).await?;
    let response = Diagram {
        components: Vec::new(),
        edges: Vec::new(),
    };
    Ok(Json(response))
}
