use axum::response::Json;
use dal::change_set::ChangeSet;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ChangeSetError,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier")
    ),
    tag = "change_sets",
    responses(
        (status = 200, description = "Change sets listed successfully", body = GetChangeSetV1Response),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_change_set(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    _tracker: PosthogEventTracker,
) -> Result<Json<GetChangeSetV1Response>, ChangeSetError> {
    let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

    Ok(Json(GetChangeSetV1Response { change_set }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetV1Response {
    #[schema(value_type = Object, example = json!({"id": "01FXNV4P306V3KGZ73YSVN8A60", "name": "My new feature"}))]
    pub change_set: ChangeSet,
}
