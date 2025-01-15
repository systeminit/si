use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt as _};
use derive_more::{Deref, Into};

use crate::app_state::AppState;

use super::{
    workspace::{TargetWorkspaceIdFromToken, WorkspaceAuthorization},
    ErrorResponse,
};

#[derive(Clone, Debug, Deref, Into)]
pub struct AccessBuilder(pub dal::AccessBuilder);

#[async_trait]
impl FromRequestParts<AppState> for AccessBuilder {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Ensure we get the workspace ID from the token
        let _: TargetWorkspaceIdFromToken = parts.extract_with_state(state).await?;
        let WorkspaceAuthorization { ctx, .. } = parts.extract_with_state(state).await?;
        Ok(Self(ctx.access_builder()))
    }
}
