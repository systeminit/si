use edda_server::{
    compressed_request::CompressedRequest,
    compressing_stream::CompressingStream,
    extract::EddaRequestKind,
};
use futures::StreamExt as _;
use test_log::test;

use self::helpers::*;

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
        .map(EddaRequestKind::Update)
        .collect();

    publish_requests(&context, test_name, requests).await;

    let mut messages = CompressingStream::new(
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

    let compressed_request: CompressedRequest =
        serde_json::from_slice(&compressed_message.payload).expect("failed to deserialize request");

    match compressed_request {
        CompressedRequest::Update {
            from_snapshot_address,
            to_snapshot_address,
            change_batch_addresses,
        } => {
            let first_from = updates.first().unwrap().from_snapshot_address;
            let last_to = updates.last().unwrap().to_snapshot_address;
            let addresses: Vec<_> = updates.iter().map(|r| r.change_batch_address).collect();

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
        .map(EddaRequestKind::Update)
        .collect();
    // Insert a rebuild into the stream of updates, mid-list
    requests.insert(4, EddaRequestKind::Rebuild(rebuild_request()));

    publish_requests(&context, test_name, requests).await;

    let mut messages = CompressingStream::new(
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

    let compressed_request: CompressedRequest =
        serde_json::from_slice(&compressed_message.payload).expect("failed to deserialize request");

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
    use std::{
        collections::HashMap,
        env,
    };

    use bytes::Bytes;
    use dal::WorkspaceSnapshotAddress;
    use edda_core::api_types::{
        ApiWrapper,
        ContentInfo,
        RequestId,
        rebuild_request::{
            RebuildRequest,
            RebuildRequestVCurrent,
        },
        update_request::{
            UpdateRequest,
            UpdateRequestV1,
        },
    };
    use edda_server::extract::EddaRequestKind;
    use futures::TryStreamExt as _;
    use nats_std::subject;
    use rand::RngCore;
    use si_data_nats::{
        HeaderMap,
        NatsClient,
        NatsConfig,
        Subject,
        async_nats::jetstream::{
            self,
            consumer::push,
        },
        jetstream::Context,
    };
    use si_events::change_batch::ChangeBatchAddress;
    use uuid::Uuid;

    const STREAM_NAME: &str = "TEST_COMPRESSING_STREAM";
    const SUBJECT_PREFIX: &str = "test.compressing_stream";

    fn nats_config(subject_prefix: String) -> NatsConfig {
        let mut config = NatsConfig::default();
        #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
        if let Ok(value) = env::var("SI_TEST_NATS_URL") {
            config.url = value;
        }
        config.subject_prefix = Some(subject_prefix);
        config
    }

    pub async fn nats(subject_prefix: String) -> NatsClient {
        NatsClient::new(&nats_config(subject_prefix))
            .await
            .expect("failed to connect to NATS")
    }

    pub fn nats_prefix() -> String {
        Uuid::new_v4().as_simple().to_string()
    }

    pub fn context(nats: NatsClient) -> Context {
        si_data_nats::jetstream::new(nats)
    }

    pub async fn stream(context: &Context) -> jetstream::stream::Stream {
        context
            .get_or_create_stream(jetstream::stream::Config {
                name: STREAM_NAME.to_string(),
                description: Some("CompressingStream integration tests".to_string()),
                subjects: vec![
                    subject::prefixed(Some(&format!("{SUBJECT_PREFIX}.*")), ">").to_string(),
                ],
                allow_direct: true,
                retention: jetstream::stream::RetentionPolicy::Limits,
                storage: jetstream::stream::StorageType::Memory,
                ..Default::default()
            })
            .await
            .expect("failed to get or create jetstream stream")
    }

    pub fn pub_sub_subject<'a>(
        prefix: impl Into<Option<&'a str>>,
        subject_suffix: &'a str,
    ) -> Subject {
        let prefix = prefix.into().map(|p| format!("{SUBJECT_PREFIX}.{p}"));
        subject::prefixed(prefix.as_deref(), subject_suffix)
    }

    pub async fn message_count_on_subject(
        stream: &jetstream::stream::Stream,
        subject: &Subject,
    ) -> usize {
        let info: HashMap<_, _> = stream
            .info_with_subjects(subject.as_str())
            .await
            .expect("failed to get stream info")
            .try_collect()
            .await
            .expect("failed to collect stream info");

        info.get(subject.as_str()).copied().unwrap_or(0)
    }

    pub async fn publish_requests(
        context: &Context,
        subject_suffix: &str,
        requests: Vec<EddaRequestKind>,
    ) {
        for request in requests {
            let (info, payload) = match request {
                EddaRequestKind::NewChangeSet(request) => {
                    let info = ContentInfo::from(&request);
                    let payload: Bytes = request
                        .to_vec()
                        .expect("failed to serialize request")
                        .into();

                    (info, payload)
                }
                EddaRequestKind::Update(request) => {
                    let info = ContentInfo::from(&request);
                    let payload: Bytes = request
                        .to_vec()
                        .expect("failed to serialize request")
                        .into();

                    (info, payload)
                }
                EddaRequestKind::Rebuild(request) => {
                    let info = ContentInfo::from(&request);
                    let payload: Bytes = request
                        .to_vec()
                        .expect("failed to serialize request")
                        .into();

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

    pub async fn incoming_messages(
        nats: &NatsClient,
        stream: &jetstream::stream::Stream,
        test_name: &str,
    ) -> push::Ordered {
        stream
            .create_consumer(push::OrderedConfig {
                deliver_subject: nats.new_inbox(),
                filter_subject: pub_sub_subject(nats.metadata().subject_prefix(), test_name)
                    .to_string(),
                ..Default::default()
            })
            .await
            .expect("failed to create consumer")
            .messages()
            .await
            .expect("failed to subscribe")
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
        UpdateRequest::new_current(UpdateRequestV1 {
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
