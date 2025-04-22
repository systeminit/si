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
use serde::{
    Deserialize,
    Serialize,
};
use si_jwt_public_key::SiJwt;

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WhoamiResponse {
    pub user_id: UserPk,
    pub user_email: String,
    pub workspace_id: WorkspacePk,
    pub token: SiJwt,
}

async fn whoami(
    // TODO this isn't really necessary; we don't need to check you have a workspace ID.
    _: TargetWorkspaceIdFromToken,
    // Just because this is the most permissive role we have right now
    _: AuthorizedForAutomationRole,
    ValidatedToken(token): ValidatedToken,
    WorkspaceAuthorization {
        workspace_id, user, ..
    }: WorkspaceAuthorization,
) -> impl IntoResponse {
    Json(WhoamiResponse {
        workspace_id,
        user_id: user.pk(),
        user_email: user.email().clone(),
        token,
    })
}
