use std::{collections::HashMap, fmt};

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
    Json,
};
use dal::{
    context::{self, DalContextBuilder},
    User, UserClaim,
};
use derive_more::Deref;
use hyper::StatusCode;

use crate::app_state::AppState;

pub struct AccessBuilder(pub context::AccessBuilder);

#[async_trait]
impl FromRequestParts<AppState> for AccessBuilder {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request_parts(parts, state).await?;
        let Tenancy(tenancy) = tenancy_from_claim(&claim).await?;

        Ok(Self(context::AccessBuilder::new(
            tenancy,
            dal::HistoryActor::from(claim.user_pk),
        )))
    }
}

/// An access builder for admin-only routes in sdf. Verifies the email for the
/// user is @systeminit.com during construction. This should only be used as a
/// route middleware for the admin routes. Use the normal AccessBuilder
/// extractor for the actual route
pub struct AdminAccessBuilder;

#[async_trait]
impl FromRequestParts<AppState> for AdminAccessBuilder {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request_parts(parts, state).await?;
        let Tenancy(tenancy) = tenancy_from_claim(&claim).await?;
        let history_actor = dal::HistoryActor::from(claim.user_pk);
        let access_builder = context::AccessBuilder::new(tenancy, history_actor);

        let dal_context_builder = state
            .services_context()
            .clone()
            .into_inner()
            .into_builder(state.for_tests());

        let ctx = dal_context_builder
            .build_head(access_builder)
            .await
            .map_err(|err| {
                let error_message =
                    format!("Unable to build dal context for head in AdminAccessBuilder: {err}");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "admin_access_builder_error",
                        "message": error_message,
                    })),
                )
            })?;

        let is_system_init = history_actor
            .email_is_systeminit(&ctx)
            .await
            .map_err(|err| {
                let error_message =
                    format!("Unable to check email address in AdminAccessBuilder: {err}");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "admin_access_builder_error_email_check",
                        "message": error_message,
                    })),
                )
            })?;

        if !is_system_init {
            Err(unauthorized_error())
        } else {
            Ok(Self)
        }
    }
}

pub struct RawAccessToken(pub String);

#[async_trait]
impl FromRequestParts<AppState> for RawAccessToken {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let raw_token_header = &parts
            .headers
            .get("Authorization")
            .ok_or_else(unauthorized_error)?;

        let full_raw_token = raw_token_header
            .to_str()
            .map_err(|_| unauthorized_error())?;

        // token looks like "Bearer asdf" so we strip off the "bearer"
        let raw_token = full_raw_token
            .split(' ')
            .last()
            .ok_or_else(unauthorized_error)?;

        Ok(Self(raw_token.to_owned()))
    }
}

pub struct HandlerContext(pub DalContextBuilder);

#[async_trait]
impl FromRequestParts<AppState> for HandlerContext {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let builder = state
            .services_context()
            .clone()
            .into_inner()
            .into_builder(state.for_tests());
        Ok(Self(builder))
    }
}

#[derive(Deref)]
pub struct AssetSprayer(pub asset_sprayer::AssetSprayer);

#[async_trait]
impl FromRequestParts<AppState> for AssetSprayer {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let asset_sprayer = state
            .asset_sprayer()
            .ok_or(not_found_error("openai not configured"))?;
        Ok(Self(asset_sprayer.clone()))
    }
}

pub struct PosthogClient(pub crate::app_state::PosthogClient);

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

pub struct Nats(pub si_data_nats::NatsClient);

#[async_trait]
impl FromRequestParts<AppState> for Nats {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let services_context = state.services_context();
        Ok(Self(services_context.nats_conn().clone()))
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
        if let Some(claim) = parts.extensions.get::<UserClaim>() {
            return Ok(Self(*claim));
        }

        let HandlerContext(builder) = HandlerContext::from_request_parts(parts, state).await?;
        let mut ctx = builder.build_default().await.map_err(internal_error)?;
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
        ctx.update_tenancy(dal::Tenancy::new(claim.workspace_pk));

        let is_authorized = User::authorize(&ctx, &claim.user_pk, &claim.workspace_pk)
            .await
            .map_err(|_| unauthorized_error())?;

        if !is_authorized {
            return Err(unauthorized_error());
        }

        parts.extensions.insert(claim);

        Ok(Self(claim))
    }
}

pub struct WsAuthorization(pub UserClaim);

#[async_trait]
impl FromRequestParts<AppState> for WsAuthorization {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let HandlerContext(builder) = HandlerContext::from_request_parts(parts, state).await?;
        let mut ctx = builder.build_default().await.map_err(internal_error)?;
        let jwt_public_signing_key = state.jwt_public_signing_key().clone();

        let query: Query<HashMap<String, String>> = Query::from_request_parts(parts, state)
            .await
            .map_err(|_| unauthorized_error())?;
        let authorization = query.get("token").ok_or_else(unauthorized_error)?;

        let claim = UserClaim::from_bearer_token(jwt_public_signing_key, authorization)
            .await
            .map_err(|_| unauthorized_error())?;
        ctx.update_tenancy(dal::Tenancy::new(claim.workspace_pk));

        let is_authorized = User::authorize(&ctx, &claim.user_pk, &claim.workspace_pk)
            .await
            .map_err(|_| unauthorized_error())?;

        if !is_authorized {
            return Err(unauthorized_error());
        }

        Ok(Self(claim))
    }
}

pub struct Tenancy(pub dal::Tenancy);

#[async_trait]
impl FromRequestParts<AppState> for Tenancy {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request_parts(parts, state).await?;
        tenancy_from_claim(&claim).await
    }
}

async fn tenancy_from_claim(
    claim: &UserClaim,
) -> Result<Tenancy, (StatusCode, Json<serde_json::Value>)> {
    Ok(Tenancy(dal::Tenancy::new(claim.workspace_pk)))
}

/// Use instead of [`axum::extract::Query`] when the query contains array params using "[]"
/// notation.
///
/// Inspiration : https://dev.to/pongsakornsemsuwan/rust-axum-extracting-query-param-of-vec-4pdm
pub struct QueryWithVecParams<T>(pub T);

#[async_trait]
impl<T> FromRequestParts<AppState> for QueryWithVecParams<T>
where
    T: serde::de::DeserializeOwned,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let query = parts
            .uri
            .query()
            .ok_or(not_found_error("no query string found in uri"))?;
        let deserialized = serde_qs::from_str(query).map_err(|err| {
            unprocessable_entity_error(&format!(
                "could not deserialize query string: {query} (error: {err})"
            ))
        })?;
        Ok(Self(deserialized))
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

pub fn unauthorized_error() -> (StatusCode, Json<serde_json::Value>) {
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

fn not_found_error(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::NOT_FOUND;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message,
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

fn unprocessable_entity_error(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::UNPROCESSABLE_ENTITY;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message,
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}
