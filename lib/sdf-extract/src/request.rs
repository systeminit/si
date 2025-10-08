use axum::{
    RequestPartsExt as _,
    async_trait,
    extract::{
        FromRequestParts,
        Query,
    },
    http::request::Parts,
};
use derive_more::{
    Deref,
    Into,
};
use sdf_core::app_state::AppState;
use serde::Deserialize;
use si_events::{
    AuthenticationMethod,
    authentication_method::AuthenticationMethodV1,
};
use si_jwt_public_key::{
    JwtKeyResult,
    SiJwt,
    SiJwtClaimRole,
    SiJwtClaims,
    validate_raw_token,
};
use ulid::Ulid;

use super::{
    AuthApiClient,
    ErrorResponse,
    internal_error,
    unauthorized_error,
};

#[derive(Clone, Debug, Deref, Into)]
pub struct RequestUlidFromHeader(pub Option<Ulid>);

#[async_trait]
impl<S> FromRequestParts<S> for RequestUlidFromHeader {
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let request_ulid = parts
            .headers
            .get("X-SI-REQUEST-ULID")
            .and_then(|u| u.to_str().ok())
            .and_then(|u| Ulid::from_string(u).ok());

        Ok(Self(request_ulid))
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

impl ValidatedToken {
    pub fn history_actor(&self) -> si_db::HistoryActor {
        si_db::HistoryActor::from(self.0.custom.user_id())
    }
    pub fn authentication_method(&self) -> JwtKeyResult<AuthenticationMethod> {
        let role = self.0.custom.role().into();
        let token_id = SiJwtClaims::token_id(&self.0)?;
        Ok(AuthenticationMethodV1::Jwt { role, token_id })
    }
}

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

        let RawAccessToken(raw_token) = parts.extract().await?;

        let jwt_public_signing_key = state.jwt_public_signing_key_chain().clone();
        let token = validate_raw_token(jwt_public_signing_key, raw_token)
            .await
            .map_err(unauthorized_error)?;

        // If it has role: Automation, check with auth API if the token has been revoked
        // (web tokens are not revocable, so we don't check them.)
        // if token.custom.role() != SiJwtClaimRole::Web {
        //     let AuthApiClient(client) = parts.extract_with_state(state).await?;
        //     // status will throw 401 if the token is revoked
        //     client.status().await.map_err(unauthorized_error)?;
        // }

        parts.extensions.insert(Self(token.clone()));
        Ok(Self(token))
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
pub struct HistoryActor(pub si_db::HistoryActor);

#[async_trait]
impl FromRequestParts<AppState> for HistoryActor {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ValidatedToken(token) = parts.extract_with_state(state).await?;
        let user_id = token.custom.user_id();

        let actor = si_db::HistoryActor::from(user_id);

        Ok(Self(actor))
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
impl<S> FromRequestParts<S> for RawAccessToken {
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(RawAccessToken(token)) = parts.extensions.get::<RawAccessToken>() {
            return Ok(Self(token.clone()));
        }

        let TokenFromAuthorizationHeader(token) = parts.extract().await?;
        Ok(Self(token))
    }
}

/// Gets the access token from the Authorization: header and strips the "Bearer" prefix
#[derive(Clone, Debug, Deref, Into)]
pub struct TokenFromAuthorizationHeader(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for TokenFromAuthorizationHeader {
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
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
