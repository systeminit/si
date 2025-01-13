use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use derive_more::{Deref, Into};

use crate::app_state::AppState;

use super::{not_found_error, ErrorResponse};

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
