use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    result,
};

use dal::SchemaId;
use edda_core::api_types::{
    Container,
    ContentInfo,
    Negotiate,
    NegotiateError,
    rebuild_changed_definitions_request::RebuildChangedDefinitionsRequest,
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
    RebuildChangedDefinitions(RebuildChangedDefinitionsRequest),
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
            RebuildChangedDefinitionsRequest::MESSAGE_TYPE => {
                Ok(DeploymentRequest::RebuildChangedDefinitions(
                    RebuildChangedDefinitionsRequest::negotiate(content_info, bytes)?,
                ))
            }
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
#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum CompressedDeploymentRequest {
    Rebuild {
        src_requests_count: usize,
    },
    RebuildChangedDefinitions {
        src_requests_count: usize,
    },
    RebuildSpecific {
        src_requests_count: usize,
        removed_schema_ids: Vec<SchemaId>,
        new_modules: HashMap<CachedModuleId, SchemaId>,
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
                info!("Compressed deployment requests {:?}", compressed);
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
            Self::RebuildSpecific {
                src_requests_count,
                removed_schema_ids: _,
                new_modules: _,
            } => *src_requests_count,
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
            let has_rebuild_for_specific = requests
                .iter()
                .any(|r| matches!(r, DeploymentRequest::RebuildSpecific(_)));

            match (
                has_rebuild,
                has_rebuild_changed_definitions,
                has_rebuild_for_specific,
            ) {
                // If we have any full rebuild requests, prioritize those (full rebuild will also fix out-of-sync definitions)
                (true, _, _) => {
                    // `cr_tc02_`
                    // `cr_tc03_`
                    // `cr_tc06_`
                    Ok(Self::Rebuild { src_requests_count })
                }
                // If we only have changed definitions rebuild requests
                (false, true, false) => {
                    // `cr_tc04_`
                    Ok(Self::RebuildChangedDefinitions { src_requests_count })
                }
                // If we only have specific rebuild requests, merge them into a single one
                (false, false, true) => {
                    // `cr_tc07_`
                    Self::all_specific_updates(requests, src_requests_count)
                }
                // If we have a mix of changed definitions and specific rebuild requests, return the specific
                (false, true, true) => {
                    // `cr_tc08_`
                    // `cr_tc09_`
                    // `cr_tc10_`
                    Self::all_specific_updates(requests, src_requests_count)
                }
                // This case shouldn't happen since we've already checked for empty requests
                (false, false, false) => unreachable!(),
            }
        }
    }

    fn all_specific_updates(
        requests: VecDeque<DeploymentRequest>,
        src_requests_count: usize,
    ) -> Result<CompressedDeploymentRequest> {
        // Collect all removed schema_ids and new module_ids from the requests
        let mut removed_schema_ids: HashSet<SchemaId> = HashSet::new();
        let mut new_modules: HashMap<CachedModuleId, SchemaId> = HashMap::new();

        for request in requests {
            match request {
                DeploymentRequest::RebuildSpecific(inner) => {
                    match inner {
                        RebuildSpecificRequest::V1(inner_v1) => {
                            // When processing removals, check if we've previously added a module that contains
                            // any of the removed schema ids--if so, remove that module from the new_modules list
                            for removed_schema_id in &inner_v1.removed_schema_ids {
                                // if we've already said this is a new module/schema, and now we've said to remove it
                                // add to the removed list and remove the module from the list to build
                                if new_modules
                                    .values()
                                    .any(|schema_id| schema_id == removed_schema_id)
                                {
                                    new_modules
                                        .retain(|_, schema_id| schema_id != removed_schema_id);
                                }
                                removed_schema_ids.insert(*removed_schema_id);
                            }

                            // When processing new modules, check if any of their schema ids are in the removed list--if so,
                            // remove them from the list to remove as we do in fact want that Id built
                            for (module_id, new_schema_id) in &inner_v1.new_modules {
                                // check if we previously said to remove this schema. If so, remove that removal :)
                                if removed_schema_ids.contains(new_schema_id) {
                                    removed_schema_ids.remove(new_schema_id);
                                }
                                new_modules.insert(*module_id, *new_schema_id);
                            }
                        }
                    }
                }
                DeploymentRequest::RebuildChangedDefinitions(_) | DeploymentRequest::Rebuild(_) => {
                    continue;
                } // Ignore other types
            }
        }

        Ok(Self::RebuildSpecific {
            src_requests_count,
            removed_schema_ids: removed_schema_ids.iter().copied().collect(),
            new_modules,
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

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc07_only_specific_rebuilds() {
        let first_input_schema_ids = vec![SchemaId::new()];
        let first_input_module_ids = HashMap::new();
        let second_input_schema_ids = vec![SchemaId::new()];
        let second_input_module_ids =
            HashMap::from_iter([(CachedModuleId::new(), SchemaId::new())]);

        let inputs = vec![
            rebuild_specific_request(
                first_input_schema_ids.clone(),
                first_input_module_ids.clone(),
            ),
            rebuild_specific_request(
                second_input_schema_ids.clone(),
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
            CompressedDeploymentRequest::RebuildChangedDefinitions {
                src_requests_count: _,
            } => {
                panic!("there should no changed definitions rebuild requests");
            }
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count,
                removed_schema_ids: mut schema_ids,
                new_modules: module_ids,
            } => {
                let mut all_input_schema_ids = Vec::new();
                all_input_schema_ids.extend(first_input_schema_ids);
                all_input_schema_ids.extend(second_input_schema_ids);

                let mut all_input_module_ids = HashMap::new();
                all_input_module_ids.extend(first_input_module_ids);
                all_input_module_ids.extend(second_input_module_ids);

                // Sort the vecs!
                all_input_schema_ids.sort();
                schema_ids.sort();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(all_input_schema_ids, schema_ids);
                assert_eq!(all_input_module_ids, module_ids);
            }
        }
    }
    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc08_mixed_definition_specific_rebuilds() {
        let first_input_schema_ids = vec![SchemaId::new()];
        let first_input_module_ids = HashMap::new();
        let second_input_schema_ids = vec![SchemaId::new()];
        let second_input_module_ids =
            HashMap::from_iter([(CachedModuleId::new(), SchemaId::new())]);
        let requests = vec![
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                first_input_schema_ids.clone(),
                first_input_module_ids.clone(),
            )),
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                second_input_schema_ids.clone(),
                second_input_module_ids.clone(),
            )),
            DeploymentRequest::RebuildChangedDefinitions(rebuild_changed_definitions_request()),
        ];

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::Rebuild {
                src_requests_count: _,
            } => {
                panic!("there should no mass rebuild requests");
            }
            CompressedDeploymentRequest::RebuildChangedDefinitions {
                src_requests_count: _,
            } => {
                panic!("there should no changed definitions rebuild requests");
            }
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count,
                removed_schema_ids: mut schema_ids,
                new_modules: module_ids,
            } => {
                let mut all_input_schema_ids = Vec::new();
                all_input_schema_ids.extend(first_input_schema_ids);
                all_input_schema_ids.extend(second_input_schema_ids);

                let mut all_input_module_ids = HashMap::new();
                all_input_module_ids.extend(first_input_module_ids);
                all_input_module_ids.extend(second_input_module_ids);

                // Sort the vecs!
                all_input_schema_ids.sort();
                schema_ids.sort();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(all_input_schema_ids, schema_ids);
                assert_eq!(all_input_module_ids, module_ids);
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc09_many_duplicate_specific_rebuilds() {
        // Create shared IDs that will be duplicated across requests
        let shared_removed_schema_id1 = SchemaId::new();
        let shared_removed_schema_id2 = SchemaId::new();
        let shared_module_id1 = CachedModuleId::new();
        let shared_module_id2 = CachedModuleId::new();
        let shared_schema_id1 = SchemaId::new();
        let shared_schema_id2 = SchemaId::new();

        // Create unique IDs for each request
        let unique_removed_schema_id1 = SchemaId::new();
        let unique_removed_schema_id2 = SchemaId::new();
        let unique_module_id1 = CachedModuleId::new();
        let unique_module_id2 = CachedModuleId::new();
        let unique_schema_id1 = SchemaId::new();
        let unique_schema_id2 = SchemaId::new();

        let requests = vec![
            // Request 1: Mix of shared and unique removed schemas/variants, and new modules
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![shared_removed_schema_id1, unique_removed_schema_id1],
                HashMap::from_iter([
                    (shared_module_id1, shared_schema_id1),
                    (unique_module_id1, unique_schema_id1),
                ]),
            )),
            // Request 2: Overlapping with request 1, plus new unique IDs
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![
                    shared_removed_schema_id1,
                    shared_removed_schema_id2,
                    unique_removed_schema_id2,
                ],
                HashMap::from_iter([
                    (shared_module_id1, shared_schema_id1), // Duplicate module
                    (shared_module_id2, shared_schema_id2),
                ]),
            )),
            // Request 3: All shared IDs (should all be duplicates)
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![shared_removed_schema_id1, shared_removed_schema_id2],
                HashMap::from_iter([
                    (shared_module_id1, shared_schema_id1),
                    (shared_module_id2, shared_schema_id2),
                ]),
            )),
            // Request 4: Mix with one more unique ID
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![shared_removed_schema_id2],
                HashMap::from_iter([(unique_module_id2, unique_schema_id2)]),
            )),
        ];

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count,
                mut removed_schema_ids,
                new_modules,
            } => {
                // Expected deduplicated results
                let mut expected_removed_schema_ids = vec![
                    shared_removed_schema_id1,
                    shared_removed_schema_id2,
                    unique_removed_schema_id1,
                    unique_removed_schema_id2,
                ];
                let mut expected_new_modules: Vec<(CachedModuleId, SchemaId)> = vec![
                    (shared_module_id1, shared_schema_id1),
                    (shared_module_id2, shared_schema_id2),
                    (unique_module_id1, unique_schema_id1),
                    (unique_module_id2, unique_schema_id2),
                ];

                // Sort for comparison
                expected_removed_schema_ids.sort();
                removed_schema_ids.sort();
                expected_new_modules.sort();
                let mut new_modules_vec: Vec<(CachedModuleId, SchemaId)> =
                    new_modules.into_iter().collect();
                new_modules_vec.sort();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(
                    expected_removed_schema_ids, removed_schema_ids,
                    "Removed schema IDs should be properly deduplicated"
                );
                assert_eq!(
                    expected_new_modules, new_modules_vec,
                    "New modules should be properly deduplicated"
                );
            }
            _ => panic!("Expected RebuildSpecific variant for specific rebuild requests"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc10_conflicting_specific_rebuilds() {
        // Create schema IDs that will be in conflict scenarios
        let schema_id_a = SchemaId::new();
        let schema_id_b = SchemaId::new();
        let schema_id_c = SchemaId::new();
        let schema_id_d = SchemaId::new(); // New schema for addition → removal test

        let module_id_1 = CachedModuleId::new();
        let module_id_2 = CachedModuleId::new();
        let module_id_3 = CachedModuleId::new();
        let module_id_4 = CachedModuleId::new();

        let requests = vec![
            // Request 1: Remove schema_id_a and its variants, add new modules for schema_id_b and schema_id_d
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![schema_id_a], // Remove schema A
                HashMap::from_iter([
                    (module_id_1, schema_id_b), // Add module for schema B
                    (module_id_4, schema_id_d), // Add module for schema D
                ]),
            )),
            // Request 2: Add new modules for schema_id_a (which was previously marked for removal) and schema_id_c
            // This should CANCEL the removal of schema_id_a from request 1
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![], // No removals in this request
                HashMap::from_iter([
                    (module_id_2, schema_id_a), // Add module for schema A (conflicts with removal!)
                    (module_id_3, schema_id_c), // Add module for schema C
                ]),
            )),
            // Request 3: Try to remove schema_id_b (which has a module from request 1)
            // This should CANCEL the addition of module for schema_id_b from request 1
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![schema_id_b], // Remove schema B
                HashMap::new(),    // No new modules
            )),
            // Request 4: Remove schema_id_d (which was added in request 1) - Addition → Removal scenario
            // This should CANCEL the addition of module for schema_id_d from request 1
            DeploymentRequest::RebuildSpecific(rebuild_specific_request(
                vec![schema_id_d], // Remove schema D
                HashMap::new(),    // No new modules
            )),
        ];

        let compressed = CompressedDeploymentRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count,
                removed_schema_ids,
                new_modules,
            } => {
                // Build the expected final request shape after conflict resolution
                // Expected results after conflict resolution:
                // - schema_id_a: Was removed in req1, but added back in req2 -> should NOT be in removed list
                // - schema_id_b: Module added in req1, but schema removed in req3 -> schema should be removed
                //   and module addition canceled
                // - schema_id_d: Module added in req1, but schema removed in req4 -> schema should be removed
                // - New modules: should contain modules for schema_id_a and schema_id_c only
                //   (modules for schema_id_b and schema_id_d should be canceled by their respective removals)

                let expected_request = CompressedDeploymentRequest::RebuildSpecific {
                    src_requests_count: requests.len(),
                    removed_schema_ids: {
                        let mut ids = vec![schema_id_b, schema_id_d];
                        ids.sort();
                        ids
                    },
                    new_modules: HashMap::from_iter([
                        (module_id_2, schema_id_a),
                        (module_id_3, schema_id_c),
                    ]),
                };

                // Construct the actual result in the same format for comparison
                let actual_request = CompressedDeploymentRequest::RebuildSpecific {
                    src_requests_count,
                    removed_schema_ids: {
                        let mut ids = removed_schema_ids;
                        ids.sort();
                        ids
                    },
                    new_modules,
                };

                // Assert that the computed result matches our expected shape exactly
                assert_eq!(
                    actual_request, expected_request,
                    "Computed request should match expected shape after conflict resolution"
                );
            }
            _ => panic!("Expected RebuildSpecific variant for specific rebuild requests"),
        }
    }

    mod helpers {
        use std::collections::HashMap;

        use dal::SchemaId;
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

        pub fn rebuild_changed_definitions_request() -> RebuildChangedDefinitionsRequest {
            RebuildChangedDefinitionsRequest::new(RebuildChangedDefinitionsRequestV1 {
                id: RequestId::new(),
            })
        }

        pub fn rebuild_specific_request(
            removed_schema_ids: Vec<SchemaId>,
            new_modules: HashMap<CachedModuleId, SchemaId>,
        ) -> RebuildSpecificRequest {
            RebuildSpecificRequest::new(RebuildSpecificRequestVCurrent {
                id: RequestId::new(),
                removed_schema_ids,
                new_modules,
            })
        }
    }
}
