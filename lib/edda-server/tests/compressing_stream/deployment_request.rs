use edda_server::{
    api_types::deployment_request::{
        CompressedDeploymentRequest,
        DeploymentRequest,
    },
    compressing_stream::CompressingStream,
};
use futures::StreamExt as _;
use test_log::test;

use self::helpers::*;
use super::super::helpers::*;

#[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
async fn multiple_rebuilds() {
    let test_name = "multiple_rebuilds";

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

    let updates = vec![
        rebuild_request(),
        rebuild_request(),
        rebuild_request(),
        rebuild_request(),
    ];
    let requests: Vec<_> = updates
        .clone()
        .into_iter()
        .map(DeploymentRequest::Rebuild)
        .collect();

    publish_requests(&context, test_name, requests.clone()).await;

    let mut messages: CompressingStream<_, _, CompressedDeploymentRequest> = CompressingStream::new(
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

    let compressed_request: CompressedDeploymentRequest =
        serde_json::from_slice(&compressed_message.payload).expect("failed to deserialize request");

    match compressed_request {
        CompressedDeploymentRequest::Rebuild { src_requests_count } => {
            assert_eq!(requests.len(), src_requests_count);
        }
    }
}

mod helpers {
    use bytes::Bytes;
    use edda_core::api_types::{
        ContentInfo,
        SerializeContainer,
    };
    use edda_server::api_types::deployment_request::DeploymentRequest;
    use si_data_nats::{
        HeaderMap,
        jetstream::Context,
    };

    use super::super::super::helpers::*;

    pub async fn publish_requests(
        context: &Context,
        subject_suffix: &str,
        requests: Vec<DeploymentRequest>,
    ) {
        for request in requests {
            let (info, payload) = match request {
                DeploymentRequest::Rebuild(request) => {
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
}
