use reqwest::Url;
use thiserror::Error;

pub use innit_core::*;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum InnitClientError {
    #[error("Deserialization error: {0}")]
    Deserialization(serde_json::Error),
    #[error("Request error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("module not found (module id: {0})")]
    ModuleNotFound(String),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(serde_json::Error),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type InnitClientResult<T> = Result<T, InnitClientError>;

#[derive(Debug, Clone)]
pub struct InnitClient {
    base_url: Url,
}

impl InnitClient {
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    pub async fn get_parameter(&self) -> InnitClientResult<GetParameterResponse> {
        let url = self.base_url.join("parameter")?;
        let resp = reqwest::Client::new()
            .get(url)
            .send()
            .await?
            .error_for_status()?;

        let parameter = resp.json::<GetParameterResponse>().await?;

        Ok(parameter)
    }

    pub async fn get_parameters_by_path(
        &self,
        path: String,
    ) -> InnitClientResult<ListParametersResponse> {
        let url = self.base_url.join("parameters")?.join(&path)?;
        let resp = reqwest::Client::new()
            .get(url)
            .send()
            .await?
            .error_for_status()?;

        let parameters = resp.json::<ListParametersResponse>().await?;

        Ok(parameters)
    }
}
