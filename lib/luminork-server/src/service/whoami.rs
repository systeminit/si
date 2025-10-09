use axum::{
    Json,
    Router,
    response::IntoResponse,
    routing::get,
};
use dal::{
    UserPk,
    WorkspacePk,
};
use sdf_extract::PosthogEventTracker;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_jwt_public_key::SiJwt;
use utoipa::ToSchema;

use crate::{
    AppState,
    extract::{
        request::ValidatedToken,
        workspace::{
            AuthorizedForAutomationRole,
            TargetWorkspaceIdFromToken,
            WorkspaceAuthorization,
        },
    },
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(whoami))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiResponse {
    #[schema(value_type = String, example = "01H9ZQCBJ3E7HBTRN3J58JQX8K")]
    pub user_id: UserPk,

    #[schema(example = "user@example.com")]
    pub user_email: String,

    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub workspace_id: WorkspacePk,

    #[schema(value_type = Object,  example = "{\"iat\":\"1746915092\",\"exp\":\"1747001492\",\"sub\":\"01GW0KXH4YJBWC7BTBAZ6ZR7EA\",\"jti\":\"01JTY41T273V8KAP9E01MWRRCN\",\"version\":\"2\",\"userId\":\"01GW0KXH4YJBWC7BTBAZ6ZR7EA\",\"workspaceId\": \"01JCKRDPB7XATNMS25FKQ1WGZZ\",\"role\": \"automation\"}")]
    pub token: SiJwt,
}

#[utoipa::path(
    get,
    path = "/whoami",
    tag = "whoami",
    responses(
        (status = 200, description = "Successfully retrieved user information", body = WhoamiResponse),
        (status = 401, description = "Unauthorized - Invalid or expired token"),
        (status = 403, description = "Forbidden - Insufficient permissions")
    ),
)]
pub async fn whoami(
    _: TargetWorkspaceIdFromToken,
    _: AuthorizedForAutomationRole,
    ValidatedToken(token): ValidatedToken,
    tracker: PosthogEventTracker,
    WorkspaceAuthorization {
        workspace_id, user, ..
    }: WorkspaceAuthorization,
) -> impl IntoResponse {
    tracker.track_no_ctx_workspace(
        workspace_id,
        "api_whoami",
        json!({
            "user_id": user.pk(),
            "user_email": user.email().clone(),
        }),
    );
    Json(WhoamiResponse {
        workspace_id,
        user_id: user.pk(),
        user_email: user.email().clone(),
        token,
    })
}
