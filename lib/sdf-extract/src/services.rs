use axum::{
    RequestPartsExt as _,
    async_trait,
    extract::{
        FromRequestParts,
        Host,
        OriginalUri,
    },
    http::{
        Uri,
        request::Parts,
    },
};
use dal::DalContext;
use derive_more::{
    Deref,
    Into,
};
use sdf_core::app_state::AppState;

use super::{
    ErrorResponse,
    internal_error,
    request::RawAccessToken,
};

#[derive(Clone, Debug, Deref, Into)]
pub struct HandlerContext(pub dal::DalContextBuilder);

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
pub struct PosthogClient(pub sdf_core::app_state::PosthogClient);

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
/// Provides a DalContext and a track() method to log the endpoint call.
///
/// Always used as part of an Authorization object (cannot be constructed).
///
#[derive(Clone)]
pub struct PosthogEventTracker {
    // These last three are so endpoints can do request tracking (they all do it the same way)
    pub posthog_client: sdf_core::app_state::PosthogClient,
    pub original_uri: Uri,
    pub host: String,
}

impl PosthogEventTracker {
    pub fn track(
        &self,
        ctx: &DalContext,
        event_name: impl AsRef<str>,
        properties: serde_json::Value,
    ) {
        sdf_core::tracking::track(
            &self.posthog_client,
            ctx,
            &self.original_uri,
            &self.host,
            event_name,
            properties,
        )
    }
}

#[async_trait]
impl FromRequestParts<AppState> for PosthogEventTracker {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Grab a few other things everybody needs (for tracking)
        let OriginalUri(original_uri) = parts.extract().await.map_err(internal_error)?;
        let Host(host) = parts.extract().await.map_err(internal_error)?;
        let PosthogClient(posthog_client) = parts.extract_with_state(state).await?;
        Ok(Self {
            posthog_client,
            original_uri,
            host,
        })
    }
}

/// An Auth API client using the same token we got from the request.
#[derive(Clone, Debug, Deref, Into)]
pub struct AuthApiClient(pub auth_api_client::client::AuthApiClient);

#[async_trait]
impl FromRequestParts<AppState> for AuthApiClient {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let RawAccessToken(raw_access_token) = parts.extract().await?;
        let client = auth_api_client::client::AuthApiClient::from_raw_token(
            state.auth_api_url(),
            raw_access_token,
        )
        .map_err(internal_error)?;
        Ok(Self(client))
    }
}

#[derive(Clone, Debug, Deref, Into)]
pub struct FriggStore(pub frigg::FriggStore);

#[async_trait]
impl FromRequestParts<AppState> for FriggStore {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.frigg().clone()))
    }
}

#[derive(Clone, Debug, Deref, Into)]
pub struct EddaClient(pub edda_client::EddaClient);

#[async_trait]
impl FromRequestParts<AppState> for EddaClient {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.edda_client().clone()))
    }
}

#[derive(Clone, Debug, Deref, Into)]
pub struct ComputeExecutor(pub dal::DedicatedExecutor);

#[async_trait]
impl FromRequestParts<AppState> for ComputeExecutor {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let compute_executor = state.services_context().compute_executor().clone();
        Ok(Self(compute_executor))
    }
}
