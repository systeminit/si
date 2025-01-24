use axum::{extract::Path, Json};
use dal::{UserPk, WorkspacePk};
use serde::{Deserialize, Serialize};

use crate::{extract::HandlerContext, service::v2::AccessBuilder};

use super::WorkspaceAPIError;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserPk,
    pub name: String,
    pub email: String,
}

impl From<dal::User> for User {
    fn from(value: dal::User) -> Self {
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

    let users = dal::User::list_members_for_workspace(&ctx, workspace_id.to_string())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(Response { users }))
}
