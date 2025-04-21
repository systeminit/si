use axum::response::Json;
use dal::change_set::ChangeSet;
use serde::Serialize;
use utoipa::ToSchema;

use crate::extract::{PosthogEventTracker, workspace::WorkspaceDalContext};

use crate::service::v1::ChangeSetError;

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets",
    params(
        ("workspace_id", description = "Workspace identifier")
    ),
    tag = "change_sets",
    responses(
        (status = 200, description = "Change sets listed successfully", body = ListChangeSetV1Response),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_change_sets(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    _tracker: PosthogEventTracker,
) -> Result<Json<ListChangeSetV1Response>, ChangeSetError> {
    let change_sets = ChangeSet::list_active(ctx).await?;

    Ok(Json(ListChangeSetV1Response { change_sets }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListChangeSetV1Response {
    #[schema(value_type = Vec<Object>, example = "[{\"id\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"name\":\"Add new feature\",\"status\":\"Draft\"}]")]
    pub change_sets: Vec<ChangeSet>,
}
