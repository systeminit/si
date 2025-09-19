use std::{
    collections::VecDeque,
    result,
};

use edda_core::api_types::{
    Container,
    ContentInfo,
    Negotiate,
    NegotiateError,
    rebuild_changed_definitions_request::RebuildChangedDefinitionsRequest,
    rebuild_request::RebuildRequest,
};
use naxum::async_trait;
use serde::{
    Deserialize,
    Serialize,
};
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
    RebuildChangedDefinitions(RebuildChangedDefinitionsRequest),
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
            RebuildChangedDefinitionsRequest::MESSAGE_TYPE => {
                Ok(DeploymentRequest::RebuildChangedDefinitions(
                    RebuildChangedDefinitionsRequest::negotiate(content_info, bytes)?,
                ))
            }
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
    RebuildChangedDefinitions { src_requests_count: usize },
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
            Self::RebuildChangedDefinitions { src_requests_count } => *src_requests_count,
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
        // Analyze what types of requests we have and compress accordingly
        else {
            let has_rebuild = requests
                .iter()
                .any(|r| matches!(r, DeploymentRequest::Rebuild(_)));
            let has_rebuild_changed_definitions = requests
                .iter()
                .any(|r| matches!(r, DeploymentRequest::RebuildChangedDefinitions(_)));

            match (has_rebuild, has_rebuild_changed_definitions) {
                // If we have any full rebuild requests, prioritize those (full rebuild will also fix out-of-sync definitions)
                (true, _) => {
                    // `cr_tc02_`
                    // `cr_tc03_`
                    Ok(Self::Rebuild { src_requests_count })
                }
                // If we only have changed definitions rebuild requests
                (false, true) => {
                    // `cr_tc04_`
                    Ok(Self::RebuildChangedDefinitions { src_requests_count })
                }
                // This case shouldn't happen since we've already checked for empty requests
                (false, false) => unreachable!(),
            }
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
            _ => panic!("expected Rebuild variant"),
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
            _ => panic!("expected Rebuild variant"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc04_single_rebuild_changed_definitions() {
        let inputs = vec![rebuild_changed_definitions_request()];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(DeploymentRequest::RebuildChangedDefinitions)
            .collect();

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::RebuildChangedDefinitions { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("expected RebuildChangedDefinitions variant"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc05_multiple_rebuild_changed_definitions() {
        let inputs = vec![
            rebuild_changed_definitions_request(),
            rebuild_changed_definitions_request(),
            rebuild_changed_definitions_request(),
        ];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(DeploymentRequest::RebuildChangedDefinitions)
            .collect();

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::RebuildChangedDefinitions { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("expected RebuildChangedDefinitions variant"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc06_mixed_requests_prioritize_full_rebuild() {
        let requests = vec![
            DeploymentRequest::RebuildChangedDefinitions(rebuild_changed_definitions_request()),
            DeploymentRequest::Rebuild(rebuild_request()),
            DeploymentRequest::RebuildChangedDefinitions(rebuild_changed_definitions_request()),
        ];

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        // Should prioritize full rebuild when mixed requests are present
        match compressed {
            CompressedDeploymentRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("expected Rebuild variant when mixed requests are present"),
        }
    }

    mod helpers {
        use edda_core::api_types::{
            Container,
            RequestId,
            rebuild_changed_definitions_request::{
                RebuildChangedDefinitionsRequest,
                RebuildChangedDefinitionsRequestV1,
            },
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

        pub fn rebuild_changed_definitions_request() -> RebuildChangedDefinitionsRequest {
            RebuildChangedDefinitionsRequest::new(RebuildChangedDefinitionsRequestV1 {
                id: RequestId::new(),
            })
        }
    }
}
