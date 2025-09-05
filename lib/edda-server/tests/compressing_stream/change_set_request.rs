use edda_server::{
    api_types::change_set_request::{
        ChangeSetRequest,
        CompressedChangeSetRequest,
    },
    compressing_stream::CompressingStream,
};
use futures::StreamExt as _;
use test_log::test;

use self::helpers::*;
use super::super::helpers::*;

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
async fn multiple_updates() {
    let test_name = "multiple_updates";

    let prefix = nats_prefix();
    let subject = pub_sub_subject(prefix.as_str(), test_name);
    let nats = nats(prefix.clone()).await;
    let context = context(nats.clone());
    let stream = stream(&context).await;

    assert_eq!(
        0,
        message_count_on_subject(&stream, &subject).await,
        "no messages for subject",
    );

    let updates = contiguous_update_requests(3);
    let requests: Vec<_> = updates
        .clone()
        .into_iter()
        .map(ChangeSetRequest::Update)
        .collect();

    publish_requests(&context, test_name, requests.clone()).await;

    let mut messages: CompressingStream<_, _, CompressedChangeSetRequest> = CompressingStream::new(
        incoming_messages(&nats, &stream, test_name).await,
        stream.clone(),
        None,
    );

    assert_eq!(
        updates.len(),
        message_count_on_subject(&stream, &subject).await,
        "messages pending processing for subject",
    );

    let compressed_message = match messages.next().await {
        Some(Ok(m)) => m,
        Some(Err(e)) => panic!("failed to sucessfully read compressed message: {e}"),
        None => panic!("failed to read compressed message, stream has closed"),
    };

    assert_eq!(subject, compressed_message.subject);

    assert_eq!(
        0,
        message_count_on_subject(&stream, &subject).await,
        "messages purged for subject",
    );

    let compressed_request: CompressedChangeSetRequest =
        serde_json::from_slice(&compressed_message.payload).expect("failed to deserialize request");

    match compressed_request {
        CompressedChangeSetRequest::Update {
            src_requests_count,
            from_snapshot_address,
            to_snapshot_address,
            change_batch_addresses,
        } => {
            let first_from = updates.first().unwrap().from_snapshot_address;
            let last_to = updates.last().unwrap().to_snapshot_address;
            let addresses: Vec<_> = updates.iter().map(|r| r.change_batch_address).collect();

            assert_eq!(requests.len(), src_requests_count);
            assert_eq!(first_from, from_snapshot_address);
            assert_eq!(last_to, to_snapshot_address);
            assert_eq!(addresses, change_batch_addresses);
        }
        _ => panic!("wrong variant for compressed request: {compressed_request:?}"),
    }
}

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
async fn updates_with_single_rebuild() {
    let test_name = "updates_with_single_rebuild";

    let prefix = nats_prefix();
    let subject = pub_sub_subject(prefix.as_str(), test_name);
    let nats = nats(prefix.clone()).await;
    let context = context(nats.clone());
    let stream = stream(&context).await;

    assert_eq!(
        0,
        message_count_on_subject(&stream, &subject).await,
        "no messages for subject",
    );

    let updates = contiguous_update_requests(5);
    let mut requests: Vec<_> = updates
        .clone()
        .into_iter()
        .map(ChangeSetRequest::Update)
        .collect();
    // Insert a rebuild into the stream of updates, mid-list
    requests.insert(4, ChangeSetRequest::Rebuild(rebuild_request()));

    publish_requests(&context, test_name, requests.clone()).await;

    let mut messages: CompressingStream<_, _, CompressedChangeSetRequest> = CompressingStream::new(
        incoming_messages(&nats, &stream, test_name).await,
        stream.clone(),
        None,
    );

    assert_eq!(
        updates.len() + 1,
        message_count_on_subject(&stream, &subject).await,
        "messages pending processing for subject",
    );

    let compressed_message = match messages.next().await {
        Some(Ok(m)) => m,
        Some(Err(e)) => panic!("failed to sucessfully read compressed message: {e}"),
        None => panic!("failed to read compressed message, stream has closed"),
    };

    assert_eq!(subject, compressed_message.subject);

    assert_eq!(
        0,
        message_count_on_subject(&stream, &subject).await,
        "messages purged for subject",
    );

    let compressed_request: CompressedChangeSetRequest =
        serde_json::from_slice(&compressed_message.payload).expect("failed to deserialize request");

    match compressed_request {
        CompressedChangeSetRequest::Rebuild { src_requests_count } => {
            assert_eq!(requests.len(), src_requests_count);
        }
        _ => panic!("wrong variant for compressed request: {compressed_request:?}"),
    }
}

mod helpers {
    use bytes::Bytes;
    use dal::WorkspaceSnapshotAddress;
    use edda_core::api_types::{
        Container,
        ContentInfo,
        RequestId,
        SerializeContainer,
        update_request::{
            UpdateRequest,
            UpdateRequestV1,
        },
    };
    use edda_server::api_types::change_set_request::ChangeSetRequest;
    use rand::RngCore;
    use si_data_nats::{
        HeaderMap,
        jetstream::Context,
    };
    use si_events::change_batch::ChangeBatchAddress;

    use super::super::super::helpers::*;

    pub async fn publish_requests(
        context: &Context,
        subject_suffix: &str,
        requests: Vec<ChangeSetRequest>,
    ) {
        for request in requests {
            let (info, payload) = match request {
                ChangeSetRequest::NewChangeSet(request) => {
                    let mut info = ContentInfo::from(&request);
                    let (content_type, payload) =
                        request.to_vec().expect("failed to serialize request");
                    info.content_type = content_type.into();
                    let payload: Bytes = payload.into();

                    (info, payload)
                }
                ChangeSetRequest::Update(request) => {
                    let mut info = ContentInfo::from(&request);
                    let (content_type, payload) =
                        request.to_vec().expect("failed to serialize request");
                    info.content_type = content_type.into();
                    let payload: Bytes = payload.into();

                    (info, payload)
                }
                ChangeSetRequest::Rebuild(request) => {
                    let mut info = ContentInfo::from(&request);
                    let (content_type, payload) =
                        request.to_vec().expect("failed to serialize request");
                    info.content_type = content_type.into();
                    let payload: Bytes = payload.into();

                    (info, payload)
                }
            };

            let mut headers = HeaderMap::new();
            info.inject_into_headers(&mut headers);

            let subject = pub_sub_subject(context.metadata().subject_prefix(), subject_suffix);

            context
                .publish_with_headers(subject, headers, payload)
                .await
                .expect("failed to publish request message")
                .await
                .expect("failed to await publish ack");
        }
    }

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

    pub fn update_request(
        from_snapshot_address: WorkspaceSnapshotAddress,
        to_snapshot_address: WorkspaceSnapshotAddress,
        change_batch_address: ChangeBatchAddress,
    ) -> UpdateRequest {
        UpdateRequest::new(UpdateRequestV1 {
            id: RequestId::new(),
            from_snapshot_address,
            to_snapshot_address,
            change_batch_address,
        })
    }

    fn rand_content() -> [u8; 32] {
        let mut buf = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut buf);
        buf
    }
}
