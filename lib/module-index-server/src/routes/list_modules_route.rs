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
    whoami::{is_systeminit_auth_token, WhoamiError},
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
    pub kind: Option<si_module::ModuleKind>,
    pub su: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesResponse {
    modules: Vec<si_module::Model>,
}

pub async fn list_module_route(
    Authorization {
        user_claim,
        auth_token,
    }: Authorization,
    DbConnection(txn): DbConnection,
    Query(request): Query<ListModulesRequest>,
    State(state): State<AppState>,
) -> Result<Json<ListModulesResponse>, ListModulesError> {
    let query = si_module::Entity::find();

    let su = request.su.unwrap_or(false)
        && is_systeminit_auth_token(&auth_token, state.token_emails()).await?;

    let kind = request.kind.unwrap_or(si_module::ModuleKind::Module);

    // filters
    let query = query
        .filter(si_module::Column::RejectedAt.is_null())
        .filter(si_module::Column::Kind.eq(kind.to_db_kind()));
    let query = if !su {
        let user_id = user_claim.user_pk.to_string();
        dbg!(&user_id);
        query.filter(si_module::Column::OwnerUserId.eq(user_id))
    } else {
        query
    };
    let query = if let Some(name_filter) = request.name {
        query.filter(si_module::Column::Name.contains(&name_filter))
    } else {
        query
    };

    // ordering
    let query = query
        .order_by_desc(si_module::Column::OwnerUserId)
        .order_by_desc(si_module::Column::CreatedAt);

    let modules: Vec<si_module::Model> = query.all(&txn).await?;

    Ok(Json(ListModulesResponse { modules }))
}
