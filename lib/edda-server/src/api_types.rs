use std::result;

use naxum::async_trait;
use thiserror::Error;

pub mod change_set_request;
pub mod deployment_request;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CompressedRequestError {
    #[error("requests list cannot be empty")]
    NoRequests,
}

type Result<T> = result::Result<T, CompressedRequestError>;

type Error = CompressedRequestError;

#[async_trait]
pub trait CompressFromRequests {
    type Request;

    async fn compress_from_requests(requests: Vec<Self::Request>) -> Result<Self>
    where
        Self: Sized;
}
