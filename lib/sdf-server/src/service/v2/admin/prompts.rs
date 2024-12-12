use anyhow::Result;
use asset_sprayer::prompt::AwsCliCommandPromptKind;
use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::prompt_override::PromptOverride;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use thiserror::Error;

use crate::{service::ApiError, AppState};

use super::AdminUserContext;

mod prompt;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum PromptAPIError {
    #[error("asset sprayer error: {0}")]
    AssetSprayer(#[from] asset_sprayer::AssetSprayerError),
    #[error("prompt override error: {0}")]
    PromptOverride(#[from] dal::prompt_override::PromptOverrideError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
}

impl IntoResponse for PromptAPIError {
    fn into_response(self) -> Response {
        let err_string = self.to_string();

        #[allow(clippy::match_single_binding)]
        let (status_code, maybe_message) = match self {
            _ => (ApiError::DEFAULT_ERROR_STATUS_CODE, None),
        };

        ApiError::new(status_code, maybe_message.unwrap_or(err_string)).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_prompts))
        .nest("/:prompt_kind", prompt::routes())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptListEntry {
    pub kind: AwsCliCommandPromptKind,
    pub overridden: bool,
}

pub async fn list_prompts(
    AdminUserContext(ctx): AdminUserContext,
) -> Result<Json<Vec<PromptListEntry>>> {
    let overrides = PromptOverride::list(&ctx).await?;
    Ok(Json(
        AwsCliCommandPromptKind::iter()
            .map(|kind| PromptListEntry {
                kind,
                overridden: overrides.contains(kind.as_ref()),
            })
            .collect(),
    ))
}
