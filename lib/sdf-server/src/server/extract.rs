use std::{collections::HashMap, fmt};

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
    Json,
};
use dal::{
    context::{self, DalContextBuilder},
    RequestContext, User, UserClaim,
};
use hyper::StatusCode;

use super::state::AppState;

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

        let raw_token = full_raw_token
            .split(" ")
            .last()
            .ok_or_else(unauthorized_error)?;

        // token looks like "Bearer asdf" so we strip off the "bearer"
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
        let builder = state.services_context().clone().into_inner().into_builder();
        Ok(Self(builder))
    }
}

pub struct PosthogClient(pub super::state::PosthogClient);

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
        let HandlerContext(builder) = HandlerContext::from_request_parts(parts, state).await?;
        let mut ctx = builder
            .build(RequestContext::default())
            .await
            .map_err(internal_error)?;
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

        User::authorize(&ctx, &claim.user_pk)
            .await
            .map_err(|_| unauthorized_error())?;

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
        let mut ctx = builder
            .build(RequestContext::default())
            .await
            .map_err(internal_error)?;
        let jwt_public_signing_key = state.jwt_public_signing_key().clone();

        let query: Query<HashMap<String, String>> = Query::from_request_parts(parts, state)
            .await
            .map_err(|_| unauthorized_error())?;
        let authorization = query.get("token").ok_or_else(unauthorized_error)?;

        let claim = UserClaim::from_bearer_token(jwt_public_signing_key, authorization)
            .await
            .map_err(|_| unauthorized_error())?;
        ctx.update_tenancy(dal::Tenancy::new(claim.workspace_pk));

        User::authorize(&ctx, &claim.user_pk)
            .await
            .map_err(|_| unauthorized_error())?;

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
