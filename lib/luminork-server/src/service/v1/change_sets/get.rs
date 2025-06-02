use axum::response::Json;
use dal::change_set::ChangeSet;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use utoipa::ToSchema;

use super::ChangeSetResult;
use crate::{
    api_types::change_sets::v1::ChangeSetViewV1,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier")
    ),
    tag = "change_sets",
    summary = "Get a Change Set by Change Set Id",
    responses(
        (status = 200, description = "Change details retrieved successfully", body = GetChangeSetV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Change Set not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_change_set(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> ChangeSetResult<Json<GetChangeSetV1Response>> {
    tracker.track(ctx, "api_get_change_set", json!({}));

    let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

    let change_set_vew = ChangeSetViewV1 {
        id: change_set.clone().id,
        name: change_set.clone().name,
        status: change_set.status,
        is_head: change_set.clone().is_head(ctx).await?,
    };

    Ok(Json(GetChangeSetV1Response {
        change_set: change_set_vew,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetV1Response {
    #[schema(example = json!({
        "id": "01FXNV4P306V3KGZ73YSVN8A60",
        "name": "My new feature",
        "status": "NeedsApproval",
        "isHead": "false"
    }))]
    pub change_set: ChangeSetViewV1,
}
