use derive_builder::UninitializedFieldError;
use reqwest::StatusCode;
use thiserror::Error;

use crate::api::PosthogMessage;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PosthogError {
    #[error("{0} must be initialized")]
    ConfigUninitializedField(&'static str),
    #[error("{0}")]
    ConfigValidationError(String),
    #[error("posthog api error: HTTP CODE {0}, BODY : {1}")]
    PosthogApi(StatusCode, String),
    #[error("properties must be a json object")]
    PropertiesType,
    #[error("http request error from reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("send error; did the api sender get die?: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<PosthogMessage>),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl From<UninitializedFieldError> for PosthogError {
    fn from(value: UninitializedFieldError) -> Self {
        Self::ConfigUninitializedField(value.field_name())
    }
}

impl From<String> for PosthogError {
    fn from(value: String) -> Self {
        Self::ConfigValidationError(value)
    }
}
pub type PosthogResult<T> = Result<T, PosthogError>;
