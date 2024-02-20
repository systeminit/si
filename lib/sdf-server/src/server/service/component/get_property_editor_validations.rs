use axum::extract::Query;
use axum::Json;
use dal::{ComponentId, PropId, StandardModel, ValidationOutput, ValidationResolver, Visibility};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPropertyEditorValidationsRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetPropertyEditorValidationsResponse =
    HashMap<PropId, Vec<(Option<String>, ValidationOutput)>>;

pub async fn get_property_editor_validations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetPropertyEditorValidationsRequest>,
) -> ComponentResult<Json<GetPropertyEditorValidationsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut validations: GetPropertyEditorValidationsResponse = HashMap::new();

    for resolver in
        ValidationResolver::find_by_attr(&ctx, "component_id", &request.component_id).await?
    {
        validations
            .entry(resolver.prop_id())
            .or_default()
            .push((resolver.key().map(ToOwned::to_owned), resolver.value()?));
    }

    Ok(Json(validations))
}
