use axum::{
    Json,
    extract::Path,
};
use dal::{
    UserPk,
    WorkspacePk,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::WorkspaceAPIError;
use crate::{
    extract::HandlerContext,
    service::v2::AccessBuilder,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserPk,
    pub name: String,
    pub email: String,
}

impl From<si_db::User> for User {
    fn from(value: si_db::User) -> Self {
        Self {
            id: value.pk(),
            name: value.name().to_owned(),
            email: value.email().to_owned(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    users: Vec<User>,
}

pub async fn list_workspace_users(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path(workspace_id): Path<WorkspacePk>,
) -> Result<Json<Response>, WorkspaceAPIError> {
    let ctx = builder.build_head(access_builder).await?;

    let users = si_db::User::list_members_for_workspace(&ctx, workspace_id.to_string())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(Response { users }))
}
