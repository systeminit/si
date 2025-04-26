use axum::response::Json;
use dal::{
    Component,
    ComponentId,
};
use serde::Serialize;
use utoipa::{
    self,
    ToSchema,
};

use super::ComponentsError;
use crate::extract::change_set::ChangeSetDalContext;

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsV1Response {
    #[schema(value_type = String)]
    pub components: Vec<ComponentId>,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
    ),
    tag = "components",
    responses(
        (status = 200, description = "Components retrieved successfully"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
) -> Result<Json<ListComponentsV1Response>, ComponentsError> {
    let component_ids = Component::list_ids(ctx).await?;

    Ok(Json(ListComponentsV1Response {
        components: component_ids,
    }))
}
