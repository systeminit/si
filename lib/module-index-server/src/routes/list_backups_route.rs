use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    app_state::AppState,
    extract::{Authorization, DbConnection},
    models::si_module,
    whoami::WhoamiError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ListModulesError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("whoami error: {0}")]
    Whoami(#[from] WhoamiError),
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for ListModulesError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesResponse {
    modules: Vec<si_module::Model>,
}

pub async fn list_backups_route(
    Authorization {
        user_claim,
        auth_token: _auth_token,
    }: Authorization,
    DbConnection(txn): DbConnection,
    Query(_request): Query<ListModulesRequest>,
    State(_state): State<AppState>,
) -> Result<Json<ListModulesResponse>, ListModulesError> {
    let query = si_module::Entity::find();

    // filter for backups from the current user only
    let query = query.filter(si_module::Column::OwnerUserId.eq(user_claim.user_pk.to_string()));
    let query = query.filter(si_module::Column::IsBackup.eq(true));

    // TODO: eventually we may want to optionally filter by workspace id, or organization, etc...

    // ordering
    let query = query.order_by_desc(si_module::Column::CreatedAt);

    // TODO: deal with paging (limit, offset)

    let modules: Vec<si_module::Model> = query.all(&txn).await?;

    Ok(Json(ListModulesResponse { modules }))
}
