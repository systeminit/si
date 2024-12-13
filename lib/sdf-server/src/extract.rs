use std::{collections::HashMap, fmt};

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
    Json,
};
use dal::{
    context::{self, DalContextBuilder},
    User,
};
use derive_more::Deref;
use hyper::StatusCode;
use si_jwt_public_key::{SiJwtClaimRole, SiJwtClaims};

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

        Ok(Self(context::AccessBuilder::new(
            dal::Tenancy::new(claim.workspace_id()),
            dal::HistoryActor::from(claim.user_id()),
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
        let AccessBuilder(access_builder) = AccessBuilder::from_request_parts(parts, state).await?;

        let dal_context_builder = state
            .services_context()
            .clone()
            .into_inner()
            .into_builder(state.for_tests());

        let head_ctx = dal_context_builder
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

        let is_system_init = access_builder
            .history_actor()
            .email_is_systeminit(&head_ctx)
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
            return Err(unauthorized_error("not admin user"));
        }
        Ok(Self)
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
            .ok_or_else(|| unauthorized_error("no Authorization header"))?;

        let full_raw_token = raw_token_header.to_str().map_err(unauthorized_error)?;

        // token looks like "Bearer asdf" so we strip off the "bearer"
        let raw_token = full_raw_token
            .strip_prefix("Bearer ")
            .ok_or_else(|| unauthorized_error("No Bearer in Authorization header"))?;

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

/** Represents a user who is authorized for the web */
#[derive(Clone, Debug)]
pub struct Authorization(pub SiJwtClaims);

#[async_trait]
impl FromRequestParts<AppState> for Authorization {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // If we already authorized this request for the web, don't do it again
        if let Some(authorization) = parts.extensions.get::<Authorization>() {
            return Ok(authorization.clone());
        }

        let HandlerContext(builder) = HandlerContext::from_request_parts(parts, state).await?;
        let mut ctx = builder.build_default().await.map_err(internal_error)?;
        let jwt_public_signing_key = state.jwt_public_signing_key_chain().clone();

        let headers = &parts.headers;
        let authorization_header_value = headers
            .get("Authorization")
            .ok_or_else(|| unauthorized_error("no Authorization header"))?;
        let authorization = authorization_header_value
            .to_str()
            .map_err(internal_error)?;
        let claim = SiJwtClaims::from_bearer_token(jwt_public_signing_key, authorization)
            .await
            .map_err(unauthorized_error)?;
        ctx.update_tenancy(dal::Tenancy::new(claim.workspace_id()));

        if !is_authorized_for(&ctx, &claim, SiJwtClaimRole::Web)
            .await
            .map_err(internal_error)?
        {
            return Err(unauthorized_error("not authorized for web role"));
        }

        parts.extensions.insert(Self);

        Ok(Self(claim))
    }
}

async fn is_authorized_for(
    ctx: &dal::DalContext,
    claim: &SiJwtClaims,
    role: SiJwtClaimRole,
) -> dal::UserResult<bool> {
    let workspace_members =
        User::list_members_for_workspace(ctx, claim.workspace_id().to_string()).await?;

    let is_member = workspace_members
        .into_iter()
        .any(|m| m.pk() == claim.user_id());

    Ok(is_member && claim.authorized_for(role))
}

pub struct WsAuthorization(pub SiJwtClaims);

#[async_trait]
impl FromRequestParts<AppState> for WsAuthorization {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let HandlerContext(builder) = HandlerContext::from_request_parts(parts, state).await?;
        let mut ctx = builder.build_default().await.map_err(internal_error)?;
        let jwt_public_signing_key = state.jwt_public_signing_key_chain().clone();

        let query: Query<HashMap<String, String>> = Query::from_request_parts(parts, state)
            .await
            .map_err(unauthorized_error)?;
        let authorization = query
            .get("token")
            .ok_or_else(|| unauthorized_error("No token in query"))?;

        let claim = SiJwtClaims::from_bearer_token(jwt_public_signing_key, authorization)
            .await
            .map_err(unauthorized_error)?;
        ctx.update_tenancy(dal::Tenancy::new(claim.workspace_id()));

        if !is_authorized_for(&ctx, &claim, SiJwtClaimRole::Web)
            .await
            .map_err(internal_error)?
        {
            return Err(unauthorized_error("not authorized for web role"));
        }

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

pub fn unauthorized_error(message: impl fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::UNAUTHORIZED;
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
