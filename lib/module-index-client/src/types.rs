use thiserror::Error;

pub mod upload;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum IndexClientError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Upload error: {0}")]
    Upload(String),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type IndexClientResult<T> = Result<T, IndexClientError>;
