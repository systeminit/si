use std::{
    collections::VecDeque,
    result,
};

use dal::{
    ChangeSetId,
    WorkspaceSnapshotAddress,
};
use edda_core::api_types::{
    Container,
    ContentInfo,
    Negotiate,
    NegotiateError,
    new_change_set_request::NewChangeSetRequest,
    rebuild_request::RebuildRequest,
    update_request::UpdateRequest,
};
use naxum::async_trait;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::change_batch::ChangeBatchAddress;
use strum::AsRefStr;
use telemetry::prelude::*;

use super::{
    CompressFromRequests,
    Error,
    Result,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChangeSetRequest {
    NewChangeSet(NewChangeSetRequest),
    Update(UpdateRequest),
    Rebuild(RebuildRequest),
}

impl Negotiate for ChangeSetRequest {
    fn negotiate(
        content_info: &ContentInfo<'_>,
        bytes: &[u8],
    ) -> result::Result<Self, NegotiateError>
    where
        Self: Sized,
    {
        match content_info.message_type.as_str() {
            UpdateRequest::MESSAGE_TYPE => Ok(ChangeSetRequest::Update(UpdateRequest::negotiate(
                content_info,
                bytes,
            )?)),
            RebuildRequest::MESSAGE_TYPE => Ok(ChangeSetRequest::Rebuild(
                RebuildRequest::negotiate(content_info, bytes)?,
            )),
            NewChangeSetRequest::MESSAGE_TYPE => Ok(ChangeSetRequest::NewChangeSet(
                NewChangeSetRequest::negotiate(content_info, bytes)?,
            )),
            unsupported => Err(NegotiateError::UnsupportedContentType(
                unsupported.to_string(),
            )),
        }
    }
}

#[remain::sorted]
#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
pub enum CompressedChangeSetRequest {
    NewChangeSet {
        src_requests_count: usize,
        base_change_set_id: ChangeSetId,
        new_change_set_id: ChangeSetId,
        to_snapshot_address: WorkspaceSnapshotAddress,
        change_batch_addresses: Vec<ChangeBatchAddress>,
    },
    // use option<index checksum> as idempotency key?
    Rebuild {
        src_requests_count: usize,
    },
    Update {
        src_requests_count: usize,
        from_snapshot_address: WorkspaceSnapshotAddress,
        to_snapshot_address: WorkspaceSnapshotAddress,
        change_batch_addresses: Vec<ChangeBatchAddress>,
    },
}

#[async_trait]
impl CompressFromRequests for CompressedChangeSetRequest {
    type Request = ChangeSetRequest;

    #[instrument(
        name = "edda.compressed_change_set_request.compress_from_requests",
        level = "debug",
        skip_all,
        fields(
            si.edda.compressed_change_set_request.inputs = Empty,
            si.edda.compressed_change_set_request.output = Empty,
        ),
    )]
    async fn compress_from_requests(requests: Vec<Self::Request>) -> Result<Self>
    where
        Self: Sized,
    {
        let span = current_span_for_instrument_at!("debug");

        if !span.is_disabled() {
            span.record(
                "si.edda.compressed_change_set_request.inputs",
                tracing::field::debug(&requests.iter().collect::<Vec<_>>()),
            );
        }

        match Self::inner_from_requests(requests).await {
            Ok(compressed) => {
                if !span.is_disabled() {
                    span.record(
                        "si.edda.compressed_change_set_request.output",
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

impl CompressedChangeSetRequest {
    pub fn src_requests_count(&self) -> usize {
        match self {
            Self::NewChangeSet {
                src_requests_count, ..
            }
            | Self::Rebuild { src_requests_count }
            | Self::Update {
                src_requests_count, ..
            } => *src_requests_count,
        }
    }

    // Note: there's an inner to help with telemetry tracking of inputs and output
    #[inline]
    async fn inner_from_requests(requests: Vec<ChangeSetRequest>) -> Result<Self> {
        let src_requests_count = requests.len();
        // Allow manipulation on front and tail of list
        let mut requests = VecDeque::from(requests);

        // If list is empty, return error--this is an invalid pre-condition for this function
        if requests.is_empty() {
            // `cr_tc01_`
            Err(Error::NoRequests)
        }
        // If all requests are new change sets, then return the first one
        else if requests
            .iter()
            .all(|request| matches!(request, ChangeSetRequest::NewChangeSet(_)))
        {
            let request = match requests.front() {
                Some(ChangeSetRequest::NewChangeSet(request)) => request,
                _ => unreachable!("vec contains at least one new change set request"),
            };

            // `cr_tc02_`
            // `cr_tc03_`
            Ok(Self::NewChangeSet {
                src_requests_count,
                base_change_set_id: request.base_change_set_id,
                new_change_set_id: request.new_change_set_id,
                to_snapshot_address: request.to_snapshot_address,
                change_batch_addresses: Vec::new(),
            })
        }
        // If all requests are rebuilds, then return a single one
        else if requests
            .iter()
            .all(|request| matches!(request, ChangeSetRequest::Rebuild(_)))
        {
            // `cr_tc04_`
            // `cr_tc05_`
            Ok(Self::Rebuild { src_requests_count })
        }
        // If all requests are updates, assert it's a contiguous series (i.e. last `to` is current
        // `from`, etc.)
        else if requests
            .iter()
            .all(|request| matches!(request, ChangeSetRequest::Update(_)))
        {
            // `cr_tc06_`
            // `cr_tc07_`
            // `cr_tc08_`
            Self::all_updates(requests, src_requests_count)
        }
        // All requests are of at least two kinds
        else {
            // Pop first element off list
            let first = requests.pop_front().expect("vec is non-empty");

            match first {
                ChangeSetRequest::NewChangeSet(first) => {
                    // If remaining list is empty, return new change set
                    if requests.is_empty() {
                        // no test as covered by above branches
                        Ok(Self::NewChangeSet {
                            src_requests_count,
                            base_change_set_id: first.base_change_set_id,
                            new_change_set_id: first.new_change_set_id,
                            to_snapshot_address: first.to_snapshot_address,
                            change_batch_addresses: vec![],
                        })
                    }
                    // If all remaining requests are updates, assert it's a contiguous series (i.e.
                    // last `to` is current `from`, etc.)
                    else if requests
                        .iter()
                        .all(|request| matches!(request, ChangeSetRequest::Update(_)))
                    {
                        match Self::all_updates(requests, src_requests_count)? {
                            // If result was new change set, return first one (note: this shouldn't
                            // be a valid code path)
                            Self::NewChangeSet { .. } => {
                                // no test as covered by above branches
                                Ok(Self::NewChangeSet {
                                    src_requests_count,
                                    base_change_set_id: first.base_change_set_id,
                                    new_change_set_id: first.new_change_set_id,
                                    to_snapshot_address: first.to_snapshot_address,
                                    change_batch_addresses: vec![],
                                })
                            }
                            // Remaining updates were discontiguous, return rebuild
                            Self::Rebuild { src_requests_count } => {
                                // `cr_tc09_`
                                Ok(Self::Rebuild { src_requests_count })
                            }
                            // Remaining updates were contigous, return new change set with updates
                            Self::Update {
                                change_batch_addresses,
                                ..
                            } => {
                                // `cr_tc10_`
                                Ok(Self::NewChangeSet {
                                    src_requests_count,
                                    base_change_set_id: first.base_change_set_id,
                                    new_change_set_id: first.new_change_set_id,
                                    to_snapshot_address: first.to_snapshot_address,
                                    change_batch_addresses,
                                })
                            }
                        }
                    }
                    // If all remainin requests are rebuilds
                    else if requests
                        .iter()
                        .all(|request| matches!(request, ChangeSetRequest::Rebuild(_)))
                    {
                        // Return new change set and drop the rebuilds.
                        //
                        // Remember: the consumer of this request will fall back to a rebuild if
                        // the index copy fails
                        //
                        // `cr_tc11_`
                        // `cr_tc12_`
                        Ok(Self::NewChangeSet {
                            src_requests_count,
                            base_change_set_id: first.base_change_set_id,
                            new_change_set_id: first.new_change_set_id,
                            to_snapshot_address: first.to_snapshot_address,
                            change_batch_addresses: vec![],
                        })
                    }
                    // Remaining requests are of at least two kinds
                    else {
                        // Filter out new change sets & rebuild and look at updates
                        requests.retain(|request| match request {
                            ChangeSetRequest::Update(_) => true,
                            ChangeSetRequest::NewChangeSet(_) | ChangeSetRequest::Rebuild(_) => {
                                false
                            }
                        });

                        if requests.is_empty() {
                            // no test as covered by above branches
                            Ok(Self::NewChangeSet {
                                src_requests_count,
                                base_change_set_id: first.base_change_set_id,
                                new_change_set_id: first.new_change_set_id,
                                to_snapshot_address: first.to_snapshot_address,
                                change_batch_addresses: vec![],
                            })
                        } else {
                            match Self::all_updates(requests, src_requests_count)? {
                                // If result was new change set, return first one (note: this
                                // shouldn't be a valid code path)
                                Self::NewChangeSet { .. } => {
                                    // no test as covered by above branches
                                    Ok(Self::NewChangeSet {
                                        src_requests_count,
                                        base_change_set_id: first.base_change_set_id,
                                        new_change_set_id: first.new_change_set_id,
                                        to_snapshot_address: first.to_snapshot_address,
                                        change_batch_addresses: vec![],
                                    })
                                }
                                // Remaining updates were discontiguous, return new change set and
                                // drop the rebuilds
                                //
                                // Remember: the consumer of this request will fall back to a
                                // rebuild if the index copy fails
                                Self::Rebuild { src_requests_count } => {
                                    // `cr_tc13_`
                                    // `cr_tc14_`
                                    Ok(Self::NewChangeSet {
                                        src_requests_count,
                                        base_change_set_id: first.base_change_set_id,
                                        new_change_set_id: first.new_change_set_id,
                                        to_snapshot_address: first.to_snapshot_address,
                                        change_batch_addresses: vec![],
                                    })
                                }
                                // Remaining updates were contigous, return new change set with
                                // updates
                                Self::Update {
                                    change_batch_addresses,
                                    ..
                                } => {
                                    // `cr_tc15_`
                                    // `cr_tc16_`
                                    // `cr_tc32_`
                                    // `cr_tc33_`
                                    Ok(Self::NewChangeSet {
                                        src_requests_count,
                                        base_change_set_id: first.base_change_set_id,
                                        new_change_set_id: first.new_change_set_id,
                                        to_snapshot_address: first.to_snapshot_address,
                                        change_batch_addresses,
                                    })
                                }
                            }
                        }
                    }
                }
                ChangeSetRequest::Rebuild(_first) => {
                    // If remaining list is empty, return rebuild
                    if requests.is_empty() {
                        // no test as covered by above branches
                        Ok(Self::Rebuild { src_requests_count })
                    }
                    // If all remaining requests are updates, assert it's a contiguous series (i.e.
                    // last `to` is current `from`, etc.)
                    else if requests
                        .iter()
                        .all(|request| matches!(request, ChangeSetRequest::Update(_)))
                    {
                        match Self::all_updates(requests, src_requests_count)? {
                            // If result was new change set, return the new change set (the
                            // rebuild/new change set requests might have arrived out of order)
                            //
                            // (note: this shouldn't be a valid code path)
                            Self::NewChangeSet {
                                src_requests_count,
                                base_change_set_id,
                                new_change_set_id,
                                to_snapshot_address,
                                change_batch_addresses,
                            } => {
                                // not reachable as a code path
                                Ok(Self::NewChangeSet {
                                    src_requests_count,
                                    base_change_set_id,
                                    new_change_set_id,
                                    to_snapshot_address,
                                    change_batch_addresses,
                                })
                            }
                            // Remaining updates were discontiguous, return rebuild
                            Self::Rebuild { src_requests_count } => {
                                // `cr_tc17_`
                                Ok(Self::Rebuild { src_requests_count })
                            }
                            // Remaining updates were contigous, but there was still a rebuild, so
                            // return rebuild
                            Self::Update {
                                src_requests_count, ..
                            } => {
                                // `cr_tc18_`
                                Ok(Self::Rebuild { src_requests_count })
                            }
                        }
                    }
                    // If all remaining requests are new change sets
                    else if requests
                        .iter()
                        .all(|request| matches!(request, ChangeSetRequest::NewChangeSet(_)))
                    {
                        // Return first new change set.
                        //
                        // Remember: the consumer of this request will fall back to a rebuild if
                        // the index copy fails
                        //
                        let first_ncsr = match requests.pop_front().expect("vec is non-empty") {
                            ChangeSetRequest::NewChangeSet(request) => request,
                            ChangeSetRequest::Update(_) | ChangeSetRequest::Rebuild(_) => {
                                unreachable!("vec popluated only with new change set requests")
                            }
                        };

                        // `cr_tc19_`
                        // `cr_tc20_`
                        Ok(Self::NewChangeSet {
                            src_requests_count,
                            base_change_set_id: first_ncsr.base_change_set_id,
                            new_change_set_id: first_ncsr.new_change_set_id,
                            to_snapshot_address: first_ncsr.to_snapshot_address,
                            change_batch_addresses: vec![],
                        })
                    }
                    // Remaining requests are of at least two kinds
                    //
                    // Now we're in to "nonsense" territory, so rebuild
                    else {
                        // `cr_tc21_`
                        // `cr_tc22_`
                        Ok(Self::Rebuild { src_requests_count })
                    }
                }
                ChangeSetRequest::Update(first) => {
                    // If remaining list is empty, return update
                    if requests.is_empty() {
                        // no test as covered by above branches
                        Ok(Self::Update {
                            src_requests_count,
                            from_snapshot_address: first.from_snapshot_address,
                            to_snapshot_address: first.to_snapshot_address,
                            change_batch_addresses: vec![first.change_batch_address],
                        })
                    }
                    // If all remaining requests are new change sets
                    else if requests
                        .iter()
                        .all(|request| matches!(request, ChangeSetRequest::NewChangeSet(_)))
                    {
                        // Return first new change set.
                        //
                        // Remember: the consumer of this request will fall back to a rebuild if
                        // the index copy fails
                        //
                        let first_ncsr = match requests.pop_front().expect("vec is non-empty") {
                            ChangeSetRequest::NewChangeSet(request) => request,
                            ChangeSetRequest::Update(_) | ChangeSetRequest::Rebuild(_) => {
                                unreachable!("vec popluated only with new change set requests")
                            }
                        };

                        // `cr_tc23_`
                        // `cr_tc24_`
                        Ok(Self::NewChangeSet {
                            src_requests_count,
                            base_change_set_id: first_ncsr.base_change_set_id,
                            new_change_set_id: first_ncsr.new_change_set_id,
                            to_snapshot_address: first_ncsr.to_snapshot_address,
                            change_batch_addresses: vec![first.change_batch_address],
                        })
                    }
                    // If all remainin requests are rebuilds
                    else if requests
                        .iter()
                        .all(|request| matches!(request, ChangeSetRequest::Rebuild(_)))
                    {
                        // `cr_tc25_`
                        // `cr_tc26_`
                        Ok(Self::Rebuild { src_requests_count })
                    }
                    // Remaining requests are of at least two kinds
                    //
                    // Now we're in to "nonsense" territory, so rebuild
                    else {
                        // `cr_tc27_`
                        // `cr_tc28_`
                        // `cr_tc29_`
                        // `cr_tc30_`
                        // `cr_tc31_`
                        Ok(Self::Rebuild { src_requests_count })
                    }
                }
            }
        }
    }

    fn all_updates(
        requests: VecDeque<ChangeSetRequest>,
        src_requests_count: usize,
    ) -> Result<CompressedChangeSetRequest> {
        let mut final_from_snapshot_address = None;
        let mut final_to_snapshot_address = None;

        let mut prev_to_snapshot_address: Option<WorkspaceSnapshotAddress> = None;

        let mut change_batch_addresses = Vec::new();

        for request in requests {
            match request {
                ChangeSetRequest::Update(request) => {
                    if final_from_snapshot_address.is_none() {
                        // Set final `from` addr from first request
                        final_from_snapshot_address = Some(request.from_snapshot_address);
                    }
                    // Last time through loop sets the final `to` addr
                    final_to_snapshot_address = Some(request.to_snapshot_address);

                    if let Some(prev_to_snapshot_address) = prev_to_snapshot_address {
                        if prev_to_snapshot_address != request.from_snapshot_address {
                            // If the previous `to` is not current `from` then there is a gap
                            // in the updates and we revert to a rebuild
                            return Ok(Self::Rebuild { src_requests_count });
                        }
                    }

                    // Update variable for next loop iteration
                    prev_to_snapshot_address = Some(request.to_snapshot_address);

                    change_batch_addresses.push(request.change_batch_address);
                }
                _ => unreachable!("vec only contains update requests"),
            }
        }

        Ok(Self::Update {
            src_requests_count,
            from_snapshot_address: final_from_snapshot_address
                .expect("option is populated in loop"),
            to_snapshot_address: final_to_snapshot_address.expect("option is populated in loop"),
            change_batch_addresses,
        })
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

        match CompressedChangeSetRequest::compress_from_requests(requests).await {
            Err(Error::NoRequests) => {
                // this is the expected error
            }
            Ok(_) => panic!("operation should error"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc02_single_new_change_set() {
        let inputs = identical_new_change_set_requests(1);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::NewChangeSet)
            .collect();

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let input = inputs.first().unwrap();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(input.base_change_set_id, base_change_set_id);
                assert_eq!(input.new_change_set_id, new_change_set_id);
                assert_eq!(input.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc03_multiple_identical_new_change_sets() {
        let inputs = identical_new_change_set_requests(4);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::NewChangeSet)
            .collect();

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let inputs = inputs.first().unwrap();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(inputs.base_change_set_id, base_change_set_id);
                assert_eq!(inputs.new_change_set_id, new_change_set_id);
                assert_eq!(inputs.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc04_single_rebuild() {
        let inputs = vec![rebuild_request()];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Rebuild)
            .collect();

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc05_multiple_rebuilds() {
        let inputs = vec![rebuild_request(), rebuild_request(), rebuild_request()];
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Rebuild)
            .collect();

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc06_single_update() {
        let inputs = contiguous_update_requests(1);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::Update {
                src_requests_count,
                from_snapshot_address,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let update = inputs.first().unwrap();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(update.from_snapshot_address, from_snapshot_address);
                assert_eq!(update.to_snapshot_address, to_snapshot_address);
                assert_eq!(vec![update.change_batch_address], change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc07_multiple_contiguous_updates() {
        let inputs = contiguous_update_requests(3);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::Update {
                src_requests_count,
                from_snapshot_address,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let first_from = inputs.first().unwrap().from_snapshot_address;
                let last_to = inputs.last().unwrap().to_snapshot_address;
                let addresses: Vec<_> = inputs.iter().map(|r| r.change_batch_address).collect();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(first_from, from_snapshot_address);
                assert_eq!(last_to, to_snapshot_address);
                assert_eq!(addresses, change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc08_multiple_discontiguous_updates() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        requests.remove(2);

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // gaps in updates should lead to a rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc09_single_new_change_set_then_multiple_discontiguous_updates() {
        let mut inputs = contiguous_update_requests(6);
        inputs.remove(2);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a new change set into the start of stream of updates
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        requests.insert(0, ChangeSetRequest::NewChangeSet(ncsr.clone()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // gaps in updates should lead to a rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc10_single_new_change_set_then_multiple_contiguous_updates() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a new change set into the start of stream of updates
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        requests.insert(0, ChangeSetRequest::NewChangeSet(ncsr.clone()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with batch addresses from updates
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let addresses: Vec<_> = inputs.iter().map(|r| r.change_batch_address).collect();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert_eq!(addresses, change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc11_single_new_change_set_then_single_rebuild() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::Rebuild(rr),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set
            // (drop rebuild. remember: the consumer of this request will fall back to a rebuild if
            // the index copy fails)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc12_single_new_change_set_then_multiple_rebuilds() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::Rebuild(rr),
            ChangeSetRequest::Rebuild(rebuild_request()),
            ChangeSetRequest::Rebuild(rebuild_request()),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set
            // (drop rebuild. remember: the consumer of this request will fall back to a rebuild if
            // the index copy fails)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc13_single_new_change_set_then_multiple_discontiguous_updates_with_rebuild() {
        let mut inputs = contiguous_update_requests(5);
        inputs.remove(2);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a new change set into the start of stream of updates
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        requests.insert(0, ChangeSetRequest::NewChangeSet(ncsr.clone()));
        // Insert a rebuild into the stream of updates
        requests.insert(3, ChangeSetRequest::Rebuild(rebuild_request()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with no update addresses
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc14_single_new_change_set_then_multiple_discontiguous_updates_with_new_change_set()
    {
        let mut inputs = contiguous_update_requests(5);
        inputs.remove(2);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a new change set into the start of stream of updates
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        requests.insert(0, ChangeSetRequest::NewChangeSet(ncsr.clone()));
        // Insert a new change set into the stream of updates
        requests.insert(3, ChangeSetRequest::NewChangeSet(ncsr.clone()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with no update addresses
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc15_single_new_change_set_then_multiple_contiguous_updates_with_rebuild() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a new change set into the start of stream of updates
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        requests.insert(0, ChangeSetRequest::NewChangeSet(ncsr.clone()));
        // Insert a rebuild into the stream of updates
        requests.insert(3, ChangeSetRequest::Rebuild(rebuild_request()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with batch address from updates
            // (filter out the second identical new change set)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let addresses: Vec<_> = inputs.iter().map(|r| r.change_batch_address).collect();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert_eq!(addresses, change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc16_single_new_change_set_then_multiple_contiguous_updates_with_new_change_set() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a new change set into the start of stream of updates
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        requests.insert(0, ChangeSetRequest::NewChangeSet(ncsr.clone()));
        // Insert a new change set into the stream of updates
        requests.insert(3, ChangeSetRequest::NewChangeSet(ncsr.clone()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with batch address from updates
            // (filter out the second identical new change set)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let addresses: Vec<_> = inputs.iter().map(|r| r.change_batch_address).collect();

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert_eq!(addresses, change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc17_single_rebuild_then_multiple_discontiguous_updates() {
        let mut inputs = contiguous_update_requests(6);
        inputs.remove(2);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a rebuild into the start of stream of updates
        requests.insert(0, ChangeSetRequest::Rebuild(rebuild_request()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // gaps in updates should lead to a rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc18_single_rebuild_then_multiple_contiguous_updates() {
        let inputs = contiguous_update_requests(6);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a rebuild into the start of stream of updates
        requests.insert(0, ChangeSetRequest::Rebuild(rebuild_request()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // rebuild even though we have contiguous updates
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc19_single_rebuild_then_single_new_change_set() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Rebuild(rr),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set
            // (drop rebuild. we'll assume this is a case with multiple clients with requests
            // coming in out of order)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc20_single_rebuild_then_multiple_new_change_sets() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Rebuild(rr),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set
            // (drop rebuild. we'll assume this is a case with multiple clients with requests
            // coming in out of order)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert!(change_batch_addresses.is_empty());
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc21_single_rebuild_then_single_new_change_set_then_single_update() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Rebuild(rr),
            ChangeSetRequest::NewChangeSet(ncsr),
            ChangeSetRequest::Update(ur),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // this is a bit nonsense, so rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc22_single_rebuild_then_single_update_then_signle_new_change_set() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Rebuild(rr),
            ChangeSetRequest::Update(ur),
            ChangeSetRequest::NewChangeSet(ncsr),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // this is a bit nonsense, so rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc23_single_update_then_single_new_change_set() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let requests = vec![
            ChangeSetRequest::Update(ur.clone()),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with the update
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert_eq!(vec![ur.change_batch_address], change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc24_single_update_then_multiple_new_change_sets() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let requests = vec![
            ChangeSetRequest::Update(ur.clone()),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with the update
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert_eq!(vec![ur.change_batch_address], change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc25_single_update_then_single_rebuild() {
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Update(ur.clone()),
            ChangeSetRequest::Rebuild(rr),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // return rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc26_single_update_then_multiple_rebuilds() {
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Update(ur.clone()),
            ChangeSetRequest::Rebuild(rr.clone()),
            ChangeSetRequest::Rebuild(rr.clone()),
            ChangeSetRequest::Rebuild(rr.clone()),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // return rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc27_multiple_contiguous_updates_with_single_rebuild() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a rebuild into the stream of updates, mid-list
        requests.push(ChangeSetRequest::Rebuild(rebuild_request()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc28_multiple_discontiguous_updates_with_single_rebuild() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert a rebuild into the stream of updates, mid-list
        requests.insert(4, ChangeSetRequest::Rebuild(rebuild_request()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc29_multiple_dicontiguous_updates_with_multiple_rebuilds() {
        let inputs = contiguous_update_requests(6);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(ChangeSetRequest::Update)
            .collect();
        // Insert rebuilds into the stream of updates, mid-list
        requests.insert(1, ChangeSetRequest::Rebuild(rebuild_request()));
        requests.insert(2, ChangeSetRequest::Rebuild(rebuild_request()));
        requests.insert(4, ChangeSetRequest::Rebuild(rebuild_request()));

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc30_single_update_then_single_new_change_set_then_single_rebuild() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Update(ur),
            ChangeSetRequest::NewChangeSet(ncsr),
            ChangeSetRequest::Rebuild(rr),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // this is a bit nonsense, so rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc31_single_update_then_single_rebuild_then_new_change_set() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::Update(ur),
            ChangeSetRequest::Rebuild(rr),
            ChangeSetRequest::NewChangeSet(ncsr),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // this is a bit nonsense, so rebuild
            CompressedChangeSetRequest::Rebuild { src_requests_count } => {
                assert_eq!(requests.len(), src_requests_count);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc32_single_new_change_set_then_single_update_then_single_rebuild() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::Update(ur.clone()),
            ChangeSetRequest::Rebuild(rr),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with updates
            // (drop rebuild)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let addresses = vec![ur.change_batch_address];

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert_eq!(addresses, change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn cr_tc33_single_new_change_set_then_single_rebuild_then_single_update() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            ChangeSetRequest::NewChangeSet(ncsr.clone()),
            ChangeSetRequest::Rebuild(rr),
            ChangeSetRequest::Update(ur.clone()),
        ];

        let compressed = CompressedChangeSetRequest::compress_from_requests(requests.clone())
            .await
            .expect("failed to compress requests");

        match compressed {
            // new change set with updates
            // (drop rebuild)
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count,
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let addresses = vec![ur.change_batch_address];

                assert_eq!(requests.len(), src_requests_count);
                assert_eq!(ncsr.base_change_set_id, base_change_set_id);
                assert_eq!(ncsr.new_change_set_id, new_change_set_id);
                assert_eq!(ncsr.to_snapshot_address, to_snapshot_address);
                assert_eq!(addresses, change_batch_addresses);
            }
            _ => panic!("wrong variant for compressed request: {compressed:?}"),
        }
    }

    mod helpers {
        use dal::{
            ChangeSetId,
            WorkspaceSnapshotAddress,
        };
        use edda_core::api_types::{
            Container,
            RequestId,
            new_change_set_request::{
                NewChangeSetRequest,
                NewChangeSetRequestVCurrent,
            },
            rebuild_request::{
                RebuildRequest,
                RebuildRequestVCurrent,
            },
            update_request::{
                UpdateRequest,
                UpdateRequestVCurrent,
            },
        };
        use rand::RngCore;
        use si_events::change_batch::ChangeBatchAddress;

        pub fn contiguous_update_requests(size: usize) -> Vec<UpdateRequest> {
            let mut requests = Vec::with_capacity(size);

            let mut from = None;

            for _i in 0..size {
                let from_snapshot_address = match from {
                    Some(from_snapshot_address) => from_snapshot_address,
                    None => WorkspaceSnapshotAddress::new(&rand_content()),
                };

                let to_snapshot_address = WorkspaceSnapshotAddress::new(&rand_content());

                from = Some(to_snapshot_address);

                let change_batch_address = ChangeBatchAddress::new(&rand_content());

                requests.push(update_request(
                    from_snapshot_address,
                    to_snapshot_address,
                    change_batch_address,
                ));
            }

            requests
        }

        pub fn identical_new_change_set_requests(size: usize) -> Vec<NewChangeSetRequest> {
            let mut requests = Vec::with_capacity(size);

            let request = new_change_set_request(
                ChangeSetId::new(),
                ChangeSetId::new(),
                WorkspaceSnapshotAddress::new(&rand_content()),
            );

            for _i in 0..size {
                requests.push(request.clone());
            }

            requests
        }

        pub fn new_change_set_request(
            base_change_set_id: ChangeSetId,
            new_change_set_id: ChangeSetId,
            to_snapshot_address: WorkspaceSnapshotAddress,
        ) -> NewChangeSetRequest {
            NewChangeSetRequest::new(NewChangeSetRequestVCurrent {
                id: RequestId::new(),
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
            })
        }

        pub fn update_request(
            from_snapshot_address: WorkspaceSnapshotAddress,
            to_snapshot_address: WorkspaceSnapshotAddress,
            change_batch_address: ChangeBatchAddress,
        ) -> UpdateRequest {
            UpdateRequest::new(UpdateRequestVCurrent {
                id: RequestId::new(),
                from_snapshot_address,
                to_snapshot_address,
                change_batch_address,
            })
        }

        pub fn rebuild_request() -> RebuildRequest {
            RebuildRequest::new(RebuildRequestVCurrent {
                id: RequestId::new(),
            })
        }

        fn rand_content() -> [u8; 32] {
            let mut buf = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut buf);
            buf
        }
    }
}
