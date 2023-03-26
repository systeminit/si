use reqwest::StatusCode;
use thiserror::Error;

use crate::api::PosthogApi;

#[derive(Error, Debug)]
pub enum PosthogError {
    #[error("http request error from reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("properties must be a json object")]
    PropertiesType,
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("posthog api error: HTTP CODE {0}, BODY : {1}")]
    PosthogApi(StatusCode, String),
    #[error("send error; did the api sender get die?: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<PosthogApi>),
}

pub type PosthogResult<T> = Result<T, PosthogError>;
