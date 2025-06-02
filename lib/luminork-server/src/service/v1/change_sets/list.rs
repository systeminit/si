use axum::response::Json;
use dal::change_set::ChangeSet;
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use super::ChangeSetResult;
use crate::{
    api_types::change_sets::v1::ChangeSetViewV1,
    extract::{
        PosthogEventTracker,
        workspace::WorkspaceDalContext,
    },
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "change_sets",
    summary = "List all active Change Sets",
    responses(
        (status = 200, description = "Change Sets listed successfully", body = ListChangeSetV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_change_sets(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
) -> ChangeSetResult<Json<ListChangeSetV1Response>> {
    tracker.track(ctx, "api_list_change_set", json!({}));

    let change_sets = ChangeSet::list_active(ctx).await?;

    let mut views: Vec<ChangeSetViewV1> = vec![];
    for change_set in change_sets {
        views.push(ChangeSetViewV1 {
            id: change_set.id,
            name: change_set.clone().name,
            status: change_set.status,
            is_head: change_set.clone().is_head(ctx).await?,
        });
    }

    Ok(Json(ListChangeSetV1Response { change_sets: views }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListChangeSetV1Response {
    #[schema(value_type = Vec<Object>, example = "[{\"id\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"name\":\"Add new feature\",\"status\":\"Open\",\"isHead\": \"false\"},{\"id\":\"01H9ZQE356JPMBGHH69BT0Q70UO\",\"name\":\"HEAD\",\"status\":\"Open\", \"isHead\": \"true\"}]")]
    pub change_sets: Vec<ChangeSetViewV1>,
}
