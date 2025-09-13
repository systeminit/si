use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    result,
};

use dal::{
    SchemaId,
    SchemaVariantId,
};
use edda_core::api_types::{
    Container,
    ContentInfo,
    Negotiate,
    NegotiateError,
    rebuild_request::RebuildRequest,
    rebuild_specific_request::RebuildSpecificRequest,
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
    RebuildSpecific(RebuildSpecificRequest),
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
            RebuildSpecificRequest::MESSAGE_TYPE => Ok(DeploymentRequest::RebuildSpecific(
                RebuildSpecificRequest::negotiate(content_info, bytes)?,
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
    Rebuild {
        src_requests_count: usize,
    },
    RebuildSpecific {
        src_requests_count: usize,
        schema_ids: Vec<SchemaId>,
        schema_variant_ids: HashMap<SchemaVariantId, SchemaId>,
        module_ids: Vec<CachedModuleId>,
    },
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
            Self::RebuildSpecific {
                src_requests_count,
                schema_ids: _,
                schema_variant_ids: _,
                module_ids: _,
            } => *src_requests_count,
        }
    }

    // Note: there's an inner to help with telemetry tracking of inputs and output
    #[inline]
    async fn inner_from_requests(requests: Vec<DeploymentRequest>) -> Result<Self> {
        let src_requests_count = requests.len();
        // Allow manipulation on front and tail of list
        let mut requests = VecDeque::from(requests);

        // If list is empty, return error--this is an invalid pre-condition for this function
        if requests.is_empty() {
            // `cr_tc01_`
            return Err(Error::NoRequests);
        }

        // Use a hash set for de-duplication.
        let mut schema_ids: HashSet<SchemaId> = HashSet::new();
        let mut schema_variant_ids: HashMap<SchemaVariantId, SchemaId> = HashMap::new();
        let mut module_ids: HashSet<CachedModuleId> = HashSet::new();

        while let Some(request) = requests.pop_front() {
            match request {
                DeploymentRequest::Rebuild(_) => {
                    // If any requests is a mass rebuild request, then return a single one.
                    return Ok(Self::Rebuild { src_requests_count });
                }
                DeploymentRequest::RebuildSpecific(inner) => match inner {
                    RebuildSpecificRequest::V1(inner_v1) => {
                        schema_ids.extend(inner_v1.schema_ids);
                        schema_variant_ids.extend(inner_v1.schema_variant_ids);
                        module_ids.extend(inner_v1.module_ids);
                    }
                },
            }
        }

        Ok(Self::RebuildSpecific {
            src_requests_count,
            schema_ids: schema_ids.iter().copied().collect(),
            schema_variant_ids,
            module_ids: module_ids.iter().copied().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use si_id::CachedModuleId;
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
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count: _,
                schema_ids: _,
                schema_variant_ids: _,
                module_ids: _,
            } => {
                panic!("there should no specific rebuild requests");
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
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count: _,
                schema_ids: _,
                schema_variant_ids: _,
                module_ids: _,
            } => {
                panic!("there should no specific rebuild requests");
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc04_single_specific_rebuild() {
        let input_schema_ids = vec![SchemaId::new()];
        let input_schema_variant_ids =
            HashMap::from_iter([(SchemaVariantId::new(), SchemaId::new())]);
        let input_module_ids = Vec::new();

        let inputs = vec![rebuild_specific_request(
            input_schema_ids.clone(),
            input_schema_variant_ids.clone(),
            input_module_ids.clone(),
        )];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(DeploymentRequest::RebuildSpecific)
            .collect();

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::Rebuild {
                src_requests_count: _,
            } => {
                panic!("there should no mass rebuild requests");
            }
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count,
                schema_ids,
                schema_variant_ids,
                module_ids,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(input_schema_ids, schema_ids);
                assert_eq!(input_schema_variant_ids, schema_variant_ids);
                assert_eq!(input_module_ids, module_ids);
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc05_multiple_specific_rebuilds() {
        let first_input_schema_ids = vec![SchemaId::new()];
        let first_input_schema_variant_ids =
            HashMap::from_iter([(SchemaVariantId::new(), SchemaId::new())]);
        let first_input_module_ids = Vec::new();
        let second_input_schema_ids = vec![SchemaId::new()];
        let second_input_schema_variant_ids =
            HashMap::from_iter([(SchemaVariantId::new(), SchemaId::new())]);
        let second_input_module_ids = vec![CachedModuleId::new()];

        let inputs = vec![
            rebuild_specific_request(
                first_input_schema_ids.clone(),
                first_input_schema_variant_ids.clone(),
                first_input_module_ids.clone(),
            ),
            rebuild_specific_request(
                second_input_schema_ids.clone(),
                second_input_schema_variant_ids.clone(),
                second_input_module_ids.clone(),
            ),
        ];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(DeploymentRequest::RebuildSpecific)
            .collect();

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::Rebuild {
                src_requests_count: _,
            } => {
                panic!("there should no mass rebuild requests");
            }
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count,
                mut schema_ids,
                schema_variant_ids,
                mut module_ids,
            } => {
                let mut all_input_schema_ids = Vec::new();
                all_input_schema_ids.extend(first_input_schema_ids);
                all_input_schema_ids.extend(second_input_schema_ids);

                let mut all_input_schema_variant_ids = HashMap::new();
                all_input_schema_variant_ids.extend(first_input_schema_variant_ids);
                all_input_schema_variant_ids.extend(second_input_schema_variant_ids);

                let mut all_input_module_ids = Vec::new();
                all_input_module_ids.extend(first_input_module_ids);
                all_input_module_ids.extend(second_input_module_ids);

                // Sort the vecs!
                all_input_schema_ids.sort();
                schema_ids.sort();
                all_input_module_ids.sort();
                module_ids.sort();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(all_input_schema_ids, schema_ids);
                assert_eq!(all_input_schema_variant_ids, schema_variant_ids);
                assert_eq!(all_input_module_ids, module_ids);
            }
        }
    }

    mod helpers {
        use std::collections::HashMap;

        use dal::{
            SchemaId,
            SchemaVariantId,
        };
        use edda_core::api_types::{
            Container,
            RequestId,
            rebuild_request::{
                RebuildRequest,
                RebuildRequestVCurrent,
            },
            rebuild_specific_request::{
                RebuildSpecificRequest,
                RebuildSpecificRequestVCurrent,
            },
        };
        use si_id::CachedModuleId;

        pub fn rebuild_request() -> RebuildRequest {
            RebuildRequest::new(RebuildRequestVCurrent {
                id: RequestId::new(),
            })
        }

        pub fn rebuild_specific_request(
            schema_ids: Vec<SchemaId>,
            schema_variant_ids: HashMap<SchemaVariantId, SchemaId>,
            module_ids: Vec<CachedModuleId>,
        ) -> RebuildSpecificRequest {
            RebuildSpecificRequest::new(RebuildSpecificRequestVCurrent {
                id: RequestId::new(),
                schema_ids,
                schema_variant_ids,
                module_ids,
            })
        }
    }
}
