use std::collections::HashMap;

use axum::Json;
use dal::{AttributeValueId, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use sdf_extract::{HandlerContext, v1::AccessBuilder};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConflictsForComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ConflictsForComponentResponse =
    HashMap<AttributeValueId, si_frontend_types::ConflictWithHead>;

pub async fn conflicts_for_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(ConflictsForComponentRequest {
        component_id: _,
        visibility,
    }): Json<ConflictsForComponentRequest>,
) -> ComponentResult<Json<ConflictsForComponentResponse>> {
    let _ctx = builder.build(request_ctx.build(visibility)).await?;

    Ok(Json(HashMap::new()))
}
