use std::{
    convert::Infallible,
    error,
    sync::Arc,
};

use async_nats::Subject;
use futures::{
    StreamExt,
    stream,
};
use naxum::{
    Head,
    Json,
    extract::State,
    handler::Handler,
};
use serde::{
    Deserialize,
    Serialize,
};
use tokio::{
    signal::unix::{
        self,
        SignalKind,
    },
    sync::Notify,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use tower::ServiceBuilder;
use tracing::info;
use tracing_subscriber::{
    EnvFilter,
    Registry,
    fmt::{
        self,
        format::FmtSpan,
    },
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
};

use self::message::LocalMessage;

const TRACING_LOG_ENV_VAR: &str = "SI_LOG";
const DEFAULT_TRACING_DIRECTIVES: &str = "nats_less_messages=trace,naxum=trace,info";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MyRequest {
    id: usize,
    message: String,
    is_cool: bool,
}

#[derive(Clone, Debug)]
struct AppState {}

async fn default(
    State(_state): State<AppState>,
    // [`LoaclMessage`] impls [`MessageHead`] and so [`Head`] info can be extracted
    head: Head,
    // Message payload has been JSON-encoded so the default [`Json`] extractor works as-is
    Json(request): Json<MyRequest>,
) -> naxum::response::Result<()> {
    info!(
        subject = head.subject.as_str(),
        reply = head.reply.as_deref(),
        headers_size = head.headers.map(|headers| headers.len()),
        status = head.status.map(|status| status.as_u16()),
        description = head.description.as_ref(),
        length = head.length,
        extensions_size = head.extensions.len(),
        request = ?request,
        "processing message",
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env(TRACING_LOG_ENV_VAR)
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_TRACING_DIRECTIVES)),
        )
        .with(
            fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .pretty(),
        )
        .try_init()?;

    // Generate local `impl Stream` of a `Vec` of `LocalMessage` which has an `impl MessageHead`,
    // thus satisfying Naxum's incoming stream requirements
    let incoming = stream::iter(messages()?)
        // The iterator stream is a stream of `Option<LocalMessage>` so we convert this into a
        // stream of `Option<Result<LocalMessage, Infallible>>`
        .map(Ok::<_, Infallible>);

    // Setup a Tower `Service` stack with some middleware
    let app = ServiceBuilder::new().service(default.with_state(AppState {}));

    // Use a Tokio `TaskTracker` and `CancellationToken` to support signal handling and graceful
    // shutdown
    let tracker = TaskTracker::new();
    let token = CancellationToken::new();

    let naxum_terminated = Arc::new(Notify::new());
    let naxum_exited = naxum_terminated.clone();

    let naxum_token = token.clone();
    tracker.spawn(async move {
        info!("ready to receive messages from a locally produced channel",);
        let result = naxum::serve(incoming, app.into_make_service())
            .with_graceful_shutdown(naxum::wait_on_cancelled(naxum_token))
            .await;
        naxum_terminated.notify_one();
        result
    });

    // Create streams of `SIGINT` (i.e. `Ctrl+c`) and `SIGTERM` signals
    let mut sig_int = unix::signal(SignalKind::interrupt())?;
    let mut sig_term = unix::signal(SignalKind::terminate())?;

    // Wait until one of the signal streams gets a signal, after which we will close the task
    // tracker and cancel the token, signaling all holders of the token
    tokio::select! {
        _ = sig_int.recv() => {
            info!("received SIGINT, performing graceful shutdown");
            tracker.close();
            token.cancel();
        }
        _ = sig_term.recv() => {
            info!("received SIGTERM, performing graceful shutdown");
            tracker.close();
            token.cancel();
        }
        _ = naxum_exited.notified() => {
            info!("naxum app exited, performing graceful shutdown");
            tracker.close();
            token.cancel();
        }
    }

    // Wait for all tasks to finish
    tracker.wait().await;

    info!("graceful shutdown complete");
    Ok(())
}

fn messages() -> Result<Vec<LocalMessage>, Box<dyn error::Error>> {
    // Produce a collection of request messages
    let messages = vec![
        MyRequest {
            id: 1,
            message: "Rush".to_owned(),
            is_cool: true,
        },
        MyRequest {
            id: 2,
            message: "Fly by Night".to_owned(),
            is_cool: false,
        },
        MyRequest {
            id: 3,
            message: "Caress of Steel".to_owned(),
            is_cool: true,
        },
        MyRequest {
            id: 4,
            message: "2112".to_owned(),
            is_cool: false,
        },
        MyRequest {
            id: 5,
            message: "A Farewell to Kings".to_owned(),
            is_cool: true,
        },
    ]
    .into_iter()
    // Encode each request into a [`LocalMessage`] which implements the [`MessageHead`] trait
    .map(|request| LocalMessage {
        subject: Subject::from_utf8(format!("my.requests.{}", request.id))
            .expect("failed to parse subject"),
        headers: None,
        payload: serde_json::to_vec(&request)
            .expect("failed to serialize")
            .into(),
    })
    .collect();

    Ok(messages)
}

mod message {
    use async_nats::{
        HeaderMap,
        Subject,
    };
    use bytes::Bytes;
    use naxum::{
        Extensions,
        Head,
        MessageHead,
    };

    // A local alternative to the core/Jetstream NATS message types
    #[derive(Clone, Debug)]
    pub struct LocalMessage {
        pub subject: Subject,
        pub headers: Option<HeaderMap>,
        pub payload: Bytes,
    }

    // Implementing the [`MessageHead`] trait makes [`LocalMessage`] a valid message type for a
    // Naxum app
    impl MessageHead for LocalMessage {
        // Each message needs a subject, but this can be anything, assuming it operates within the
        // naming rules of a [`Subject`]
        fn subject(&self) -> &Subject {
            &self.subject
        }

        // This type won't have reply subjects as we don't use NATS with this type
        fn reply(&self) -> Option<&Subject> {
            None
        }

        // Headers might still be useful to propagate OpenTelemetry spans, allow for more complex
        // message encodings, etc.
        fn headers(&self) -> Option<&async_nats::HeaderMap> {
            self.headers.as_ref()
        }

        // This type won't use a status as we don't use NATS with this type
        fn status(&self) -> Option<naxum::StatusCode> {
            None
        }

        // No need for a description either, so this will always be `None`
        fn description(&self) -> Option<&str> {
            None
        }

        // It's not worth adding the extra size calculations for the message (i.e. it's not going
        // to be encoded as bytes), so return the payload size as a rough approximation
        fn length(&self) -> usize {
            self.payload_length()
        }

        // Payload is still encoded in [`Bytes`] so use its length as normal
        fn payload_length(&self) -> usize {
            self.payload.len()
        }

        fn from_head_and_payload(
            head: naxum::Head,
            payload: bytes::Bytes,
        ) -> Result<(Self, naxum::Extensions), naxum::FromPartsError>
        where
            Self: Sized,
        {
            let Head {
                subject,
                // Replies aren't tracked with this type
                reply: _,
                headers,
                // Status isn't tracked with this type
                status: _,
                // Descriptions aren't used with this type
                description: _,
                // Length isn't tracked with this type
                length: _,
                extensions,
            } = head;

            Ok((
                Self {
                    subject,
                    headers,
                    payload,
                },
                extensions,
            ))
        }

        fn into_head_and_payload(self) -> (naxum::Head, bytes::Bytes) {
            let Self {
                subject,
                headers,
                payload,
            } = self;

            (
                Head {
                    subject,
                    // Replies aren't tracked with this type
                    reply: None,
                    headers,
                    // Status isn't tracked with this type
                    status: None,
                    // Descriptions aren't used with this type
                    description: None,
                    // Length isn't tracked with this type
                    length: 0,
                    extensions: Extensions::new(),
                },
                payload,
            )
        }
    }
}
