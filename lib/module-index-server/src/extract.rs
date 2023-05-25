use std::{collections::HashMap, fmt};

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
    Json,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

use crate::jwt_key::{JwtKeyError, JwtPublicSigningKey};

use super::app_state::AppState;

pub struct PosthogClient(pub super::app_state::PosthogClient);

#[async_trait]
impl FromRequestParts<AppState> for PosthogClient {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.posthog_client().clone()))
    }
}

pub struct DbConnection(pub DatabaseTransaction);

#[async_trait]
impl FromRequestParts<AppState> for DbConnection {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match state.pg_pool().begin().await {
            Ok(txn) => Ok(DbConnection(txn)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": err.to_string(),
                        "code": 42,
                        "statusCode": 500
                    }
                })),
            )),
        }
    }
}

pub type UserPk = Ulid;
pub type WorkspacePk = Ulid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct UserClaim {
    pub user_pk: UserPk,
    pub workspace_pk: WorkspacePk,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    JwtKey(#[from] JwtKeyError),
}

pub type AuthResult<T> = Result<T, AuthError>;

impl UserClaim {
    pub fn new(user_pk: UserPk, workspace_pk: WorkspacePk) -> Self {
        UserClaim {
            user_pk,
            workspace_pk,
        }
    }

    pub async fn from_bearer_token(
        public_key: JwtPublicSigningKey,
        token: impl AsRef<str>,
    ) -> AuthResult<UserClaim> {
        let claims = crate::jwt_key::validate_bearer_token(public_key, &token).await?;
        Ok(claims.custom)
    }
}

pub struct Authorization(pub UserClaim);

#[async_trait]
impl FromRequestParts<AppState> for Authorization {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jwt_public_signing_key = state.jwt_public_signing_key().clone();

        let headers = &parts.headers;
        let authorization_header_value = headers
            .get("Authorization")
            .ok_or_else(unauthorized_error)?;
        let authorization = authorization_header_value
            .to_str()
            .map_err(internal_error)?;
        let claim = UserClaim::from_bearer_token(jwt_public_signing_key, authorization)
            .await
            .map_err(|_| unauthorized_error())?;

        Ok(Self(claim))
    }
}

fn internal_error(message: impl fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::INTERNAL_SERVER_ERROR;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

fn unauthorized_error() -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::UNAUTHORIZED;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": "unauthorized",
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}
