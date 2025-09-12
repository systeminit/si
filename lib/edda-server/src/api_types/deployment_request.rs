use std::{
    collections::VecDeque,
    result,
};

use edda_core::api_types::{
    Container,
    ContentInfo,
    Negotiate,
    NegotiateError,
    rebuild_request::RebuildRequest,
    rebuild_minimal_request::RebuildMinimalRequest,

};
use naxum::async_trait;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::CachedModuleId;
use strum::AsRefStr;
use telemetry::prelude::*;

use super::{
    CompressFromRequests,
    Error,
    Result,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeploymentRequest {
    Rebuild(RebuildRequest),
    RebuildMinimal(RebuildMinimalRequest),
}

impl Negotiate for DeploymentRequest {
    fn negotiate(
        content_info: &ContentInfo<'_>,
        bytes: &[u8],
    ) -> result::Result<Self, NegotiateError>
    where
        Self: Sized,
    {
        match content_info.message_type.as_str() {
            RebuildRequest::MESSAGE_TYPE => Ok(DeploymentRequest::Rebuild(
                RebuildRequest::negotiate(content_info, bytes)?,
            )),
            unsupported => Err(NegotiateError::UnsupportedContentType(
                unsupported.to_string(),
            )),
        }
    }
}

#[remain::sorted]
#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
pub enum CompressedDeploymentRequest {
    Rebuild { src_requests_count: usize },
    RebuildMinimal { src_requests_count: usize, new_module_ids: Vec<CachedModuleId>, removed_module_ids: Vec<CachedModuleId> },
}

#[async_trait]
impl CompressFromRequests for CompressedDeploymentRequest {
    type Request = DeploymentRequest;

    #[instrument(
        name = "edda.compressed_deployment_request.compress_from_requests",
        level = "debug",
        skip_all,
        fields(
            si.edda.compressed_deployment_request.inputs = Empty,
            si.edda.compressed_deployment_request.output = Empty,
        ),
    )]
    async fn compress_from_requests(requests: Vec<Self::Request>) -> Result<Self>
    where
        Self: Sized,
    {
        let span = current_span_for_instrument_at!("debug");

        if !span.is_disabled() {
            span.record(
                "si.edda.compressed_deployment_request.inputs",
                tracing::field::debug(&requests.iter().collect::<Vec<_>>()),
            );
        }

        match Self::inner_from_requests(requests).await {
            Ok(compressed) => {
                if !span.is_disabled() {
                    span.record(
                        "si.edda.compressed_deployment_request.output",
                        tracing::field::debug(&compressed),
                    );
                    span.record_ok();
                }
                Ok(compressed)
            }
            Err(err) => Err(span.record_err(err)),
        }
    }
}

impl CompressedDeploymentRequest {
    pub fn src_requests_count(&self) -> usize {
        match self {
            Self::Rebuild { src_requests_count } => *src_requests_count,
            Self::RebuildMinimal { src_requests_count, new_module_ids, removed_module_ids } => *src_requests_count,
        }
    }

    // Note: there's an inner to help with telemetry tracking of inputs and output
    #[inline]
    async fn inner_from_requests(requests: Vec<DeploymentRequest>) -> Result<Self> {
        let src_requests_count = requests.len();
        // Allow manipulation on front and tail of list
        let requests = VecDeque::from(requests);

        // If list is empty, return error--this is an invalid pre-condition for this function
        if requests.is_empty() {
            // `cr_tc01_`
            Err(Error::NoRequests)
        }
        // If all requests are rebuilds, then return a single one
        else {
            // `cr_tc02_`
            // `cr_tc03_`
            Ok(Self::Rebuild { src_requests_count })
        }
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use self::helpers::*;
    use super::*;

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc01_no_requests() {
        let requests = vec![];

        match CompressedDeploymentRequest::compress_from_requests(requests).await {
            Err(Error::NoRequests) => {
                // this is the expected error
            }
            Ok(_) => panic!("operation should error"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc02_single_rebuild() {
        let inputs = vec![rebuild_request()];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(DeploymentRequest::Rebuild)
            .collect();

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc03_multiple_rebuilds() {
        let inputs = vec![rebuild_request(), rebuild_request(), rebuild_request()];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(DeploymentRequest::Rebuild)
            .collect();

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
        }
    }

    mod helpers {
        use edda_core::api_types::{
            Container,
            RequestId,
            rebuild_request::{
                RebuildRequest,
                RebuildRequestVCurrent,
            },
        };

        pub fn rebuild_request() -> RebuildRequest {
            RebuildRequest::new(RebuildRequestVCurrent {
                id: RequestId::new(),
            })
        }
    }
}
