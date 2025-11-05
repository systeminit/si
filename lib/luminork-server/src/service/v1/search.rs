use std::sync::Arc;

use axum::{
    Router,
    extract::Query,
    response::Json,
    routing::get,
};
use sdf_core::app_state::AppState;
use sdf_extract::{
    FriggStore,
    change_set::ChangeSetAuthorization,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use utoipa::{
    self,
    ToSchema,
};

use crate::{
    extract::PosthogEventTracker,
    search::{
        self,
        component::ComponentSearchResult,
    },
    service::v1::ComponentsError,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(search))
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/search",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("q" = String, Query, description = "Query string. See https://docs.systeminit.com/explanation/search-syntax for details.", example = "AWS::EC2::Instance region:us-east-1")
    ),
    summary = "Complex search for components",
    responses(
        (status = 200, description = "Components retrieved successfully", body = SearchV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 412, description = "Precondition Failed - missing or invalid change set index"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn search(
    FriggStore(ref frigg): FriggStore,
    ChangeSetAuthorization {
        workspace_id,
        change_set_id,
        ..
    }: ChangeSetAuthorization,
    tracker: PosthogEventTracker,
    Query(SearchV1Request { q }): Query<SearchV1Request>,
) -> Result<Json<SearchV1Response>, ComponentsError> {
    let query = Arc::new(q.parse()?);
    let components = search::component::search(frigg, workspace_id, change_set_id, &query).await?;

    tracker.track_no_ctx(
        workspace_id,
        change_set_id,
        "api_search",
        json!({
            "q": q,
            "components": components.len(),
        }),
    );

    Ok(Json(SearchV1Response { components }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchV1Request {
    #[schema(example = "AWS::EC2::Instance region:us-east-1")]
    pub q: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchV1Response {
    #[schema(example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub components: Vec<ComponentSearchResult>,
}
