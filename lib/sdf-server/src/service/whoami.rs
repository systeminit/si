use axum::{response::IntoResponse, routing::get, Json, Router};
use dal::{UserPk, WorkspacePk};
use serde::{Deserialize, Serialize};
use si_jwt_public_key::SiJwt;

use crate::{
    extract::{AuthorizedForAutomationRole, EndpointAuthorization, ValidatedToken},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(whoami))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WhoamiResponse {
    pub user_id: UserPk,
    pub user_email: String,
    pub workspace_id: WorkspacePk,
    pub token: SiJwt,
}

async fn whoami(
    // Just because this is the most permissive role we have right now
    _: AuthorizedForAutomationRole,
    ValidatedToken(token): ValidatedToken,
    EndpointAuthorization {
        workspace_id, user, ..
    }: EndpointAuthorization,
) -> impl IntoResponse {
    Json(WhoamiResponse {
        workspace_id,
        user_id: user.pk(),
        user_email: user.email().clone(),
        token,
    })
}
