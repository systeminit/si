use std::fmt;

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
    http::StatusCode,
    Json,
};
use dal::{
    context::{self, DalContextBuilder},
    User, WorkspacePk,
};
use derive_more::{Deref, Into};
use serde::Deserialize;
use si_jwt_public_key::{validate_raw_token, SiJwt, SiJwtClaimRole};

use crate::app_state::AppState;

type ErrorResponse = (StatusCode, Json<serde_json::Value>);

/// An authorized user + workspace
#[derive(Clone, Debug, Deref, Into)]
pub struct AccessBuilder(pub context::AccessBuilder);

#[async_trait]
impl FromRequestParts<AppState> for AccessBuilder {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Ensure the endpoint is authorized
        let auth = EndpointAuthorization::from_request_parts(parts, state).await?;

        Ok(Self(context::AccessBuilder::new(
            dal::Tenancy::new(auth.workspace_id),
            dal::HistoryActor::from(auth.user.pk()),
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
    type Rejection = ErrorResponse;

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

#[derive(Clone, Debug, Deref, Into)]
pub struct HandlerContext(pub DalContextBuilder);

#[async_trait]
impl FromRequestParts<AppState> for HandlerContext {
    type Rejection = ErrorResponse;

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

#[derive(Clone, Debug, Deref, Into)]
pub struct AssetSprayer(pub asset_sprayer::AssetSprayer);

#[async_trait]
impl FromRequestParts<AppState> for AssetSprayer {
    type Rejection = ErrorResponse;

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

#[derive(Clone, Debug, Deref, Into)]
pub struct PosthogClient(pub crate::app_state::PosthogClient);

#[async_trait]
impl FromRequestParts<AppState> for PosthogClient {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.posthog_client().clone()))
    }
}

#[derive(Clone, Debug, Deref, Into)]
pub struct Nats(pub si_data_nats::NatsClient);

#[async_trait]
impl FromRequestParts<AppState> for Nats {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let services_context = state.services_context();
        Ok(Self(services_context.nats_conn().clone()))
    }
}

///
/// Handles the whole endpoint authorization (checking if the user is a member of the workspace
/// as well as checking that their token has the correct role).
///
/// Equivalent to calling both AuthorizedRole (or AuthorizedForWeb/AutomationRole) and WorkspaceMember.
///
/// Unless you have already used the `TokenParamAccessToken` extractor to get the token from
/// query parameters, this will retrieve the token from the Authorization header.
///
/// Unless you have already used the `AuthorizeForAutomationRole` extractor to check that the
/// token has the automation role, this will check for maximal permissions (the web role).
///
#[derive(Clone, Debug)]
pub struct EndpointAuthorization {
    pub user: User,
    pub workspace_id: WorkspacePk,
    pub authorized_role: SiJwtClaimRole,
}

#[async_trait]
impl FromRequestParts<AppState> for EndpointAuthorization {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let WorkspaceMember { user, workspace_id } =
            WorkspaceMember::from_request_parts(parts, state).await?;
        let AuthorizedRole(authorized_role) =
            AuthorizedRole::from_request_parts(parts, state).await?;
        Ok(Self {
            user,
            workspace_id,
            authorized_role,
        })
    }
}

///
/// A user who has been validated as a member of the workspace, but whose role has *not*
/// been checked for authorization.
///
#[derive(Clone, Debug)]
struct WorkspaceMember {
    pub user: User,
    pub workspace_id: WorkspacePk,
}

#[async_trait]
impl FromRequestParts<AppState> for WorkspaceMember {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(result) = parts.extensions.get::<Self>() {
            return Ok(result.clone());
        }

        // Get the claims from the JWT
        let token = ValidatedToken::from_request_parts(parts, state).await?.0;
        let workspace_id = token.custom.workspace_id();

        // Get a context associated with the workspace
        let HandlerContext(builder) = HandlerContext::from_request_parts(parts, state).await?;
        let mut ctx = builder.build_default().await.map_err(internal_error)?;
        ctx.update_tenancy(dal::Tenancy::new(workspace_id));

        // Check if the user is a member of the workspace (and get the record if so)
        let workspace_members = User::list_members_for_workspace(&ctx, workspace_id.to_string())
            .await
            .map_err(internal_error)?;
        let user = workspace_members
            .into_iter()
            .find(|m| m.pk() == token.custom.user_id())
            .ok_or_else(|| unauthorized_error("User not a member of the workspace"))?;

        // Stash and return the result
        let result = Self { user, workspace_id };
        parts.extensions.insert(result.clone());
        Ok(result)
    }
}

///
/// Confirms that this endpoint has been authorized for the desired role, but *not* that they
/// are .
///
/// Stores the role that was authorized.
///
/// To authorize for something other than web, use the `AuthorizeForAutomationRole` extractor.
///
/// If it has not been authorized, this requires both that the maximal permissions (the web role).
///
#[derive(Clone, Copy, Debug)]
struct AuthorizedRole(pub SiJwtClaimRole);

impl AuthorizedRole {
    async fn authorize_for(
        parts: &mut Parts,
        state: &AppState,
        role: SiJwtClaimRole,
    ) -> Result<AuthorizedRole, ErrorResponse> {
        // This must not be done twice.
        if parts.extensions.get::<AuthorizedRole>().is_some() {
            return Err(internal_error(
                "Must only specify explicit endpoint authorization once",
            ));
        }

        // Validate the token meets the role
        let token = ValidatedToken::from_request_parts(parts, state).await?.0;
        if !token.custom.authorized_for(role) {
            return Err(unauthorized_error("Not authorized for web role"));
        }

        // Stash the authorization
        parts.extensions.insert(AuthorizedRole(role));

        Ok(AuthorizedRole(role))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthorizedRole {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(&result) = parts.extensions.get::<AuthorizedRole>() {
            return Ok(result);
        }
        AuthorizedRole::authorize_for(parts, state, SiJwtClaimRole::Web).await
    }
}

///
/// A user who has been authorized for the given workspace for the web role.
///
/// Does *not* validate that the user is a member of the workspace. EndpointAuthorization
/// (and WorkspaceMember) handle that.
///
#[derive(Clone, Copy, Debug)]
pub struct AuthorizedForWebRole;

#[async_trait]
impl FromRequestParts<AppState> for AuthorizedForWebRole {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        AuthorizedRole::authorize_for(parts, state, SiJwtClaimRole::Web).await?;
        Ok(Self)
    }
}

///
/// A user who has been authorized for the given workspace for the web role.
///
#[derive(Clone, Copy, Debug)]
pub struct AuthorizedForAutomationRole;

#[async_trait]
impl FromRequestParts<AppState> for AuthorizedForAutomationRole {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        AuthorizedRole::authorize_for(parts, state, SiJwtClaimRole::Automation).await?;
        Ok(Self)
    }
}

///
/// Validated JWT with unverified claims inside.
///
/// Will retrieve this from RawAccessToken, which defaults to getting the Authorization header.
/// Use TokenParamAccessToken to get it from query parameters instead (for WS connections).
///
/// Have not checked whether the user is a member of the workspace or has permissions.
///
#[derive(Clone, Debug, Deref, Into)]
pub struct ValidatedToken(pub SiJwt);

#[async_trait]
impl FromRequestParts<AppState> for ValidatedToken {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(Self(claims)) = parts.extensions.get::<Self>() {
            return Ok(Self(claims.clone()));
        }

        let raw_token = RawAccessToken::from_request_parts(parts, state).await?.0;

        let jwt_public_signing_key = state.jwt_public_signing_key_chain().clone();
        let token = validate_raw_token(jwt_public_signing_key, raw_token)
            .await
            .map_err(unauthorized_error)?;
        parts.extensions.insert(Self(token.clone()));
        Ok(Self(token))
    }
}

/// The raw JWT token string.
///
/// If this has not been extracted from the request, it will be extracted from the
/// Authorization header.
///
/// Call TokenParamAccessToken to get the token from the query parameters (for WS connections)
#[derive(Clone, Debug, Deref, Into)]
pub struct RawAccessToken(pub String);

#[async_trait]
impl FromRequestParts<AppState> for RawAccessToken {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(RawAccessToken(token)) = parts.extensions.get::<RawAccessToken>() {
            return Ok(Self(token.clone()));
        }

        let token = TokenFromAuthorizationHeader::from_request_parts(parts, state)
            .await?
            .0;
        Ok(Self(token))
    }
}

/// Gets the access token from the Authorization: header and strips the "Bearer" prefix
#[derive(Clone, Debug, Deref, Into)]
pub struct TokenFromAuthorizationHeader(pub String);

#[async_trait]
impl FromRequestParts<AppState> for TokenFromAuthorizationHeader {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(RawAccessToken(token)) = parts.extensions.get::<RawAccessToken>() {
            return Ok(Self(token.clone()));
        }

        let raw_token_header = &parts
            .headers
            .get("Authorization")
            .ok_or_else(|| unauthorized_error("no Authorization header"))?;

        let bearer_token = raw_token_header.to_str().map_err(unauthorized_error)?;

        // token looks like "Bearer asdf" so we strip off the "bearer"
        let token = bearer_token
            .strip_prefix("Bearer ")
            .ok_or_else(|| unauthorized_error("No Bearer in Authorization header"))?
            .to_owned();

        parts.extensions.insert(RawAccessToken(token.clone()));

        Ok(Self(token))
    }
}

/// Gets the access token from the "token" query parameter and strips the "Bearer" prefix
#[derive(Clone, Debug, Deref, Into)]
pub struct TokenFromQueryParam(pub String);

#[derive(Deserialize)]
struct TokenParam {
    token: String,
}

#[async_trait]
impl FromRequestParts<AppState> for TokenFromQueryParam {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TokenParam { token } = Query::from_request_parts(parts, state)
            .await
            .map_err(unauthorized_error)?
            .0;

        // TODO there is a chance that somebody retrieved the token during the await, though
        // the headers should come *after* the params in all cases ... may need to do something
        // to force other extractors to wait (like put an awaitable rawaccesstoken instead of a
        // finished one).
        if parts.extensions.get::<RawAccessToken>().is_some() {
            return Err(internal_error("Token was already extracted!"));
        }

        // token looks like "Bearer asdf" so we strip off the "bearer"
        let token = token
            .strip_prefix("Bearer ")
            .ok_or_else(|| unauthorized_error("No Bearer in token query parameter"))?
            .to_owned();

        parts.extensions.insert(RawAccessToken(token.clone()));

        Ok(Self(token))
    }
}

fn internal_error(message: impl fmt::Display) -> ErrorResponse {
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

pub fn unauthorized_error(message: impl fmt::Display) -> ErrorResponse {
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

fn not_found_error(message: &str) -> ErrorResponse {
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
