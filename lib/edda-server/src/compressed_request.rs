use std::result;

use dal::{
    ChangeSetId,
    WorkspaceSnapshotAddress,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::change_batch::ChangeBatchAddress;
use strum::AsRefStr;
use thiserror::Error;

use crate::extract::EddaRequestKind;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CompressedRequestError {
    #[error("internal error: missing to or from snapshot addresses (this is a bug!)")]
    InternalError,
    #[error("requests list cannot be empty")]
    NoRequests,
}

type Result<T> = result::Result<T, CompressedRequestError>;

type Error = CompressedRequestError;

#[remain::sorted]
#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
pub enum CompressedRequest {
    NewChangeSet {
        base_change_set_id: ChangeSetId,
        new_change_set_id: ChangeSetId,
        to_snapshot_address: WorkspaceSnapshotAddress,
    },
    Rebuild,
    Update {
        from_snapshot_address: WorkspaceSnapshotAddress,
        to_snapshot_address: WorkspaceSnapshotAddress,
        change_batch_addresses: Vec<ChangeBatchAddress>,
    },
}

impl CompressedRequest {
    // NOTE(fnichol): this function is `async` but not currently required. Shortly, we'll likely be
    // awaiting futures in this code and as it affects the `CompressingStream` we're going to leave
    // it this way with future compat in mind.
    pub async fn from_requests(requests: Vec<EddaRequestKind>) -> Result<Self> {
        if requests.is_empty() {
            return Err(Error::NoRequests);
        }

        if requests
            .iter()
            .any(|request| matches!(request, EddaRequestKind::Rebuild(_)))
        {
            return Ok(Self::Rebuild);
        }

        let mut from_snapshot_address = None;
        let mut to_snapshot_address = None;
        let mut change_batch_addresses = Vec::new();

        for request in requests {
            match request {
                EddaRequestKind::NewChangeSet(_request) => {
                    // Self::NewChangeSet {
                    //     base_change_set_id: request.base_change_set_id,
                    //     new_change_set_id: request.new_change_set_id,
                    //     to_snapshot_address: request.to_snapshot_address,
                    // };
                    todo!("FIXME");
                }
                EddaRequestKind::Update(request) => {
                    // Set from addr if not yet set (i.e. it's the first/oldest from addr)
                    if from_snapshot_address.is_none() {
                        from_snapshot_address = Some(request.from_snapshot_address);
                    }
                    // Set to addr (last element to set this is the last/newest to addr)
                    to_snapshot_address = Some(request.to_snapshot_address);

                    change_batch_addresses.push(request.change_batch_address);
                }
                EddaRequestKind::Rebuild(_) => {
                    // We already scanned for rebuilds so this match arm should not fire
                    continue;
                }
            }
        }

        let (to_snapshot_address, from_snapshot_address) =
            match (to_snapshot_address, from_snapshot_address) {
                (Some(to), Some(from)) => (to, from),
                _ => return Err(Error::InternalError),
            };

        Ok(Self::Update {
            from_snapshot_address,
            to_snapshot_address,
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
    async fn single_new_change_set() {
        let inputs = identical_new_change_set_requests(1);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::NewChangeSet)
            .collect();

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::NewChangeSet {
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
            } => {
                let compressed = inputs.first().unwrap();

                assert_eq!(compressed.base_change_set_id, base_change_set_id);
                assert_eq!(compressed.new_change_set_id, new_change_set_id);
                assert_eq!(compressed.to_snapshot_address, to_snapshot_address);
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn multiple_identical_new_change_sets() {
        let inputs = identical_new_change_set_requests(4);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::NewChangeSet)
            .collect();

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::NewChangeSet {
                base_change_set_id,
                new_change_set_id,
                to_snapshot_address,
            } => {
                let compressed = inputs.first().unwrap();

                assert_eq!(compressed.base_change_set_id, base_change_set_id);
                assert_eq!(compressed.new_change_set_id, new_change_set_id);
                assert_eq!(compressed.to_snapshot_address, to_snapshot_address);
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_update() {
        let inputs = contiguous_update_requests(1);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Update)
            .collect();

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::Update {
                from_snapshot_address,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let update = inputs.first().unwrap();

                assert_eq!(update.from_snapshot_address, from_snapshot_address);
                assert_eq!(update.to_snapshot_address, to_snapshot_address);
                assert_eq!(vec![update.change_batch_address], change_batch_addresses);
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn multiple_updates() {
        let inputs = contiguous_update_requests(3);
        let requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Update)
            .collect();

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::Update {
                from_snapshot_address,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let first_from = inputs.first().unwrap().from_snapshot_address;
                let last_to = inputs.last().unwrap().to_snapshot_address;
                let addresses: Vec<_> = inputs.iter().map(|r| r.change_batch_address).collect();

                assert_eq!(first_from, from_snapshot_address);
                assert_eq!(last_to, to_snapshot_address);
                assert_eq!(addresses, change_batch_addresses);
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_rebuild() {
        let inputs = vec![rebuild_request()];
        let requests = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Rebuild)
            .collect();

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::Rebuild => {
                // compressed request is a rebuild
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn multiple_rebuilds() {
        let inputs = vec![rebuild_request(), rebuild_request(), rebuild_request()];
        let requests = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Rebuild)
            .collect();

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::Rebuild => {
                // compressed request is a rebuild
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn no_requests() {
        let requests = vec![];

        match CompressedRequest::from_requests(requests).await {
            Err(CompressedRequestError::NoRequests) => {
                // this is the expected error
            }
            Err(err) => panic!("wrong error expected: {err}"),
            Ok(_) => panic!("operation should error"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_new_change_set_then_multiple_updates() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Update)
            .collect();
        // Insert a new change set into the start of stream of updates
        requests.insert(
            0,
            EddaRequestKind::NewChangeSet(identical_new_change_set_requests(1).pop().unwrap()),
        );

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_new_change_set_then_multiple_updates_with_new_change_set() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Update)
            .collect();
        // Insert a new change set into the start of stream of updates and the same request
        // again mid-stream
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        requests.insert(0, EddaRequestKind::NewChangeSet(ncsr.clone()));
        requests.insert(3, EddaRequestKind::NewChangeSet(ncsr.clone()));

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_new_change_set_then_single_rebuild() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::NewChangeSet(ncsr),
            EddaRequestKind::Rebuild(rr),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_rebuild_then_single_new_change_set() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::Rebuild(rr),
            EddaRequestKind::NewChangeSet(ncsr),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_new_change_set_then_single_update_then_single_rebuild() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::NewChangeSet(ncsr),
            EddaRequestKind::Update(ur),
            EddaRequestKind::Rebuild(rr),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_new_change_set_then_single_rebuild_then_single_update() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::NewChangeSet(ncsr),
            EddaRequestKind::Rebuild(rr),
            EddaRequestKind::Update(ur),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_update_then_single_new_change_set_then_single_rebuild() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::Update(ur),
            EddaRequestKind::NewChangeSet(ncsr),
            EddaRequestKind::Rebuild(rr),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_update_then_single_rebuild_then_new_change_set() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::Update(ur),
            EddaRequestKind::Rebuild(rr),
            EddaRequestKind::NewChangeSet(ncsr),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_rebuild_then_single_new_change_set_then_single_update() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::Rebuild(rr),
            EddaRequestKind::NewChangeSet(ncsr),
            EddaRequestKind::Update(ur),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn single_rebuild_then_single_update_then_signle_new_change_set() {
        let ncsr = identical_new_change_set_requests(1).pop().unwrap();
        let ur = contiguous_update_requests(1).pop().unwrap();
        let rr = rebuild_request();
        let requests = vec![
            EddaRequestKind::Rebuild(rr),
            EddaRequestKind::Update(ur),
            EddaRequestKind::NewChangeSet(ncsr),
        ];

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        todo!("FIXME: what's the right message here?");

        match compressed_request {
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn multiple_updates_with_single_rebuild() {
        let inputs = contiguous_update_requests(5);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Update)
            .collect();
        // Insert a rebuild into the stream of updates, mid-list
        requests.insert(4, EddaRequestKind::Rebuild(rebuild_request()));

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::Rebuild => {
                // compressed request is a rebuild
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn multiple_updates_with_multiple_rebuilds() {
        let inputs = contiguous_update_requests(6);
        let mut requests: Vec<_> = inputs
            .clone()
            .into_iter()
            .map(EddaRequestKind::Update)
            .collect();
        // Insert rebuilds into the stream of updates, mid-list
        requests.insert(1, EddaRequestKind::Rebuild(rebuild_request()));
        requests.insert(2, EddaRequestKind::Rebuild(rebuild_request()));
        requests.insert(4, EddaRequestKind::Rebuild(rebuild_request()));

        let compressed_request = CompressedRequest::from_requests(requests)
            .await
            .expect("failed to compress requests");

        match compressed_request {
            CompressedRequest::Rebuild => {
                // compressed request is a rebuild
            }
            _ => panic!(
                "wrong variant for compressed request: {:?}",
                compressed_request
            ),
        }
    }

    mod helpers {
        use dal::{
            ChangeSetId,
            WorkspaceSnapshotAddress,
        };
        use edda_core::api_types::{
            ApiWrapper as _,
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
            NewChangeSetRequest::new_current(NewChangeSetRequestVCurrent {
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
            UpdateRequest::new_current(UpdateRequestVCurrent {
                id: RequestId::new(),
                from_snapshot_address,
                to_snapshot_address,
                change_batch_address,
            })
        }

        pub fn rebuild_request() -> RebuildRequest {
            RebuildRequest::new_current(RebuildRequestVCurrent {
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
