use axum::extract::Query;
use axum::Json;
use dal::validation::{Validation, ValidationOutput};
use dal::{ComponentId, PropId, Visibility};
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

    // TODO Move this to the attribute value itself. There's no reason for this to be a separate call
    for resolver in Validation::list_for_component(&ctx, &request.component_id).await? {
        validations
            .entry(resolver.prop_id())
            .or_default()
            .push((resolver.key().map(ToOwned::to_owned), resolver.value()?));
    }

    Ok(Json(validations))
}
