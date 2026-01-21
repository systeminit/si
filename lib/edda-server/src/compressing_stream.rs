use std::{
    collections::{
        HashMap,
        VecDeque,
    },
    error,
    fmt,
    mem,
    pin::Pin,
    result,
    task::{
        Context,
        Poll,
    },
    time::Instant,
};

use edda_core::api_types::{
    ContentInfo,
    ContentInfoError,
    Negotiate,
    NegotiateError,
};
use futures::{
    FutureExt,
    Stream,
    StreamExt,
    TryStreamExt,
    future::BoxFuture,
};
use naxum::MessageHead;
use pin_project_lite::pin_project;
use serde::Serialize;
use si_data_nats::{
    HeaderMap,
    Subject,
    async_nats::jetstream::{
        self,
        stream::DeleteMessageError,
    },
};
use strum::AsRefStr;
use telemetry::{
    OpenTelemetrySpanExt,
    opentelemetry::trace::{
        SpanContext,
        TraceContextExt,
    },
    prelude::*,
};
use telemetry_nats::propagation;
use telemetry_utils::{
    histogram,
    monotonic,
};
use thiserror::Error;
use tokio::sync::watch;

use crate::{
    api_types::{
        CompressFromRequests,
        CompressedRequestError,
    },
    local_message::LocalMessage,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CompressingStreamError {
    #[error("failed to get count of messages on subject {0}; setting value to `1`")]
    CalculateReadWindowCountNotFound(Subject),
    #[error("failed to compress requests; skipping requests & deleting messages: {0}")]
    CompressedRequest(CompressedRequestError),
    #[error("failed to compress requests; skipping requests, deleting messages & closing: {0}")]
    CompressedRequestBeforeClose(CompressedRequestError),
    #[error("failed to delete message from stream; skipping message & deleting remaining: {0}")]
    DeleteStreamMessage(DeleteMessageError),
    #[error("failed to delete message from stream; skipping message, & deleting remaining: {0}")]
    DeleteStreamMessageAfterCompressError(DeleteMessageError),
    #[error(
        "failed to delete message from stream; skipping message, deleting remaining & closing: {0}"
    )]
    DeleteStreamMessageAfterCompressErrorBeforeClose(DeleteMessageError),
    #[error(
        "failed to delete message from stream; skipping message, deleting remaining & closing: {0}"
    )]
    DeleteStreamMessageBeforeClose(DeleteMessageError),
    #[error("failed to parse info from first message; skipping message: {0}")]
    FirstMessageInfoParse(Box<dyn error::Error + Send + Sync + 'static>),
    #[error(
        "failed to parse info from next message; skipping message & compressing remaining: {0}"
    )]
    NextMessageInfoParse(Box<dyn error::Error + Send + Sync + 'static>),
    #[error("failed to parse api request from first message; deleting message: {0}")]
    ParseFirstRequest(NegotiateError),
    #[error(
        "failed to parse api request from next message; skipping message & compressing remaining: {0}"
    )]
    ParseNextRequestInWindow(NegotiateError),
    #[error("error on subscription stream on first read; skipping message: {0}")]
    ReadFirstMessage(Box<dyn error::Error + Send + Sync + 'static>),
    #[error(
        "error on subscription stream on next read; skipping message & compressing remaining: {0}"
    )]
    ReadNextMessageInWindow(Box<dyn error::Error + Send + Sync + 'static>),
    #[error("failed to serialize compressed request to local message: {0}")]
    SerializeLocalMessage(serde_json::Error),
    #[error("failed to serialize compressed request to local message; closing: {0}")]
    SerializeLocalMessageBeforeClose(serde_json::Error),
}

type Result<T> = result::Result<T, CompressingStreamError>;

type Error = CompressingStreamError;

/// Internal state machine of [`CompressingStream`].
#[derive(AsRefStr, Default)]
enum State<R, C> {
    #[default]
    /// 1. Reading the first message from the subscription
    ReadFirstMessage,
    /// 2. Calculating the number of messages to read, a.k.a the "read window"
    CalculateReadWindow {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The message to be parsed ([`Option`] for `mem::take()`)
        message: Box<Option<jetstream::Message>>,
        /// The stream sequence number of the first message
        message_stream_sequence: u64,
        /// A [`Future`] that calculates the read window
        calculate_read_window_fut: BoxFuture<
            'static,
            result::Result<usize, Box<dyn error::Error + Send + Sync + 'static>>,
        >,
    },
    /// 3. Parsing the first message into an API request
    ParseFirstRequest {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The number of messages to read
        read_window: usize,
        /// The stream sequence number of the first message
        message_stream_sequence: u64,
        /// A [`Future`] that parses a Jetstream message into an API request and extracts headers
        parse_message_fut:
            BoxFuture<'static, result::Result<(R, Option<HeaderMap>), NegotiateError>>,
    },
    /// 4. Reading the next message from the subscription in the read window
    ReadNextMessageInWindow {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The number of messages to read
        read_window: usize,
        /// The accumulated list of read and parsed API requests
        requests: Vec<R>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The accumulated list of span contexts from incoming messages for creating span links
        span_contexts: Vec<SpanContext>,
    },
    /// 5. Parsing the next message into an API request in the read window
    ParseNextRequestInWindow {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The number of messages to read
        read_window: usize,
        /// The stream sequence number of the initial message
        message_stream_sequence: u64,
        /// A [`Future`] that parses a Jetstream message into an API request and extracts headers
        parse_message_fut:
            BoxFuture<'static, result::Result<(R, Option<HeaderMap>), NegotiateError>>,
        /// The accumulated list of read and parsed API requests
        requests: Vec<R>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The accumulated list of span contexts from incoming messages for creating span links
        span_contexts: Vec<SpanContext>,
    },
    /// 5. Compressing multiple API requests into a single compressed request
    CompressRequests {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that compresses multiple API requests into a single "compressed" request
        compress_messages_fut: BoxFuture<'static, result::Result<C, CompressedRequestError>>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The accumulated list of span contexts from incoming messages for creating span links
        span_contexts: Vec<SpanContext>,
    },
    /// 7. Deleting a message from the Jetstream stream
    DeleteStreamMessage {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The compressed request to later yield (outer [`Option`] for `mem::take()`, and inner is
        /// when there were no requests to be compressed)
        compressed_request: Option<Option<C>>,
        /// The number of successfully deleted messages
        deleted_count: usize,
    },
    /// 8. Converting request into final message to yield from [`Stream`]
    YieldItem {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The compressed request to later yield ([`Option`] for `mem::take()`)
        compressed_request: Option<C>,
    },
    /// 3.1 Deleting the first message after error
    DeleteFirstMessageAfterError {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
    },
    /// 4.1 Compressing remaining requests before closing [`Stream`]
    CompressRequestsAndClose {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that compresses multiple API requests into a single "compressed" request
        compress_messages_fut: BoxFuture<'static, result::Result<C, CompressedRequestError>>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The accumulated list of span contexts from incoming messages for creating span links
        span_contexts: Vec<SpanContext>,
    },
    /// 4.2 Deleting messages from the Jetstream stream before closing [`Stream`]
    DeleteStreamMessageAndClose {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The compressed request to later yield (outer [`Option`] for `mem::take()`, and inner is
        /// when there were no requests to be compressed)
        compressed_request: Option<Option<C>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The number of successfully deleted messages
        deleted_count: usize,
    },
    /// 4.3. Converting request into final message to yield from [`Stream`] before closing
    /// [`Stream`]
    YieldItemAndClose {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The compressed request to later yield ([`Option`] for `mem::take()`)
        compressed_request: Option<C>,
    },
    /// 4.1.1 Closing the stream
    CloseStream,
    /// 6.1 Deleting messages from the Jetstream stream after failing to compress requests
    DeleteStreamMessageAfterCompressError {
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The number of successfully deleted messages
        deleted_count: usize,
    },
    /// 4.1.2 Deleting messages from the Jetstream stream after failing to compress requests before
    /// closing [`Stream`]
    DeleteStreamMessageAfterCompressErrorAndClose {
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The number of successfully deleted messages
        deleted_count: usize,
    },
}

pin_project! {
    pub struct CompressingStream<S, R, C> {
        #[pin]
        subscription: S,
        stream: jetstream::stream::Stream,
        state: State<R, C>,
        last_compressing_heartbeat_tx: Option<watch::Sender<Instant>>,
        span: Option<Span>,
        span_started_at: Option<Instant>,
    }
}

impl<S, R, C> CompressingStream<S, R, C> {
    /// Creates and return a new CompressingStream.
    pub fn new(
        subscription: S,
        stream: jetstream::stream::Stream,
        last_compressing_heartbeat_tx: impl Into<Option<watch::Sender<Instant>>>,
    ) -> Self {
        Self {
            subscription,
            stream,
            state: Default::default(),
            last_compressing_heartbeat_tx: last_compressing_heartbeat_tx.into(),
            span: None,
            span_started_at: None,
        }
    }
}

impl<S, R, C> fmt::Debug for CompressingStream<S, R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressingStream").finish_non_exhaustive()
    }
}

impl<S, R, C, E> Stream for CompressingStream<S, R, C>
where
    S: Stream<Item = result::Result<jetstream::Message, E>>,
    R: Negotiate + Send + 'static,
    C: Serialize + CompressFromRequests<Request = R> + AsRef<str>,
    E: error::Error + Send + Sync + 'static,
{
    type Item = Result<LocalMessage>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // Get the current span so we can set properties on it. This can be in one of two
            // states:
            //
            // - compressing_stream.idle spans represent time spent waiting for messages. It
            //   starts when callers start listening for batches (i.e. when they pull from the
            //   stream), and end when a message is received.
            // - compressing_stream.batch spans represent time spent retrieving and bundling a
            //   batch of messages. This span starts when the the caller is listening and at
            //   least one message has come in. It ends when all available messages have
            //   been pulled off the NATS queue and the batch yielded to the caller.
            //
            // If there is no current span when the user polls, that means the user has begun
            // listening, so we create a compressing_stream.idle span.
            let mut span = this.span.get_or_insert_with(|| {
                let follows_from = Span::current();

                let s = span!(
                    parent: None,
                    Level::DEBUG,
                    "compressing_stream.idle",
                    messages.deleted.count = Empty,
                    messaging.destination.name = Empty,
                    read_window.count = Empty,
                    requests.count = Empty,
                    compressed.kind = Empty,
                    task.state = Empty,
                );
                s.follows_from(follows_from);

                // Track when span started for total duration metric
                *this.span_started_at = Some(Instant::now());

                s
            });
            let mut guard = span.enter();

            match this.state {
                // 1. Reading the first message from the subscription
                State::ReadFirstMessage => {
                    // Read first message from subscription
                    let poll_start = Instant::now();
                    match this.subscription.poll_next_unpin(cx) {
                        // Read the first Jetstream message successfully
                        Poll::Ready(Some(Ok(message))) => {
                            let poll_duration = poll_start.elapsed();
                            debug!(
                                poll_duration_ms = poll_duration.as_millis(),
                                "subscription.poll_next_unpin completed for first message",
                            );

                            // Metrics: Track subscription poll latency and message count
                            histogram!(
                                compressing_stream_subscription_poll_latency_ms =
                                    poll_duration.as_millis() as f64,
                                message_position = "first"
                            );
                            monotonic!(
                                compressing_stream_messages_read = 1,
                                message_position = "first"
                            );

                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Determine the stream sequence number of this message so we can
                            // delete it later.
                            let message_stream_sequence = match message.info() {
                                // Info parsed successfully from the message
                                Ok(info) => info.stream_sequence,
                                // Failed to parse [`Info`] from message
                                Err(err) => {
                                    // We can't delete this message easily as the sequence number
                                    // comes from the [`Info`] struct, so we're going to restart
                                    // the whole process
                                    trace!(
                                        si.error.message = ?err,
                                        messaging.destination.name = message.subject.as_str(),
                                        concat!(
                                            "failed to parse Info from first message; ",
                                            "skipping message",
                                        ),
                                    );

                                    // Set next state and return error
                                    *this.state = State::ReadFirstMessage;
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    drop(guard);
                                    *this.span = None;
                                    return Poll::Ready(Some(Err(Error::FirstMessageInfoParse(
                                        err,
                                    ))));
                                }
                            };

                            // We've received a message! End the compressing_stream.idle span
                            // and start a new compressing_stream.batch span to represent the
                            // batch processing.

                            // End the idle span
                            span.record_ok();
                            drop(guard);
                            *this.span = None;

                            // Emit metric for idle wait duration
                            if let Some(span_start) = this.span_started_at.take() {
                                let idle_duration = span_start.elapsed();
                                histogram!(
                                    compressing_stream_idle_duration_ms =
                                        idle_duration.as_millis() as f64
                                );
                            }

                            // Create the new batch span
                            let follows_from = Span::current();
                            let batch_span = span!(
                                parent: None,
                                Level::INFO,
                                "compressing_stream.batch",
                                messages.deleted.count = Empty,
                                messaging.destination.name = Empty,
                                read_window.count = Empty,
                                requests.count = Empty,
                                compressed.kind = Empty,
                                task.state = Empty,
                            );
                            batch_span.follows_from(follows_from);

                            // Store the new batch span and start time
                            *this.span = Some(batch_span);
                            *this.span_started_at = Some(Instant::now());

                            // CRITICAL: Update the local span variable to point to the batch span
                            // so all subsequent span.record() calls use the batch span
                            span = this.span.as_mut().unwrap();
                            // NOTE: we don't use guard again in this codepath, but it's important
                            // to keep span and guard in sync to avoid confusion.
                            #[allow(unused_assignments)]
                            {
                                guard = span.enter();
                            }

                            // Record the messaging destination on the BATCH span (not the idle span)
                            span.record("messaging.destination.name", message.subject.as_str());

                            let subject = Some(message.subject.clone());
                            let fut_subject = message.subject.clone();

                            let stream = this.stream.clone();

                            // Set next state and continue loop
                            *this.state = State::CalculateReadWindow {
                                subject,
                                message: Box::new(Some(message)),
                                message_stream_sequence,
                                calculate_read_window_fut: Box::pin(async move {
                                    let info: HashMap<_, _> = stream
                                        .info_with_subjects(fut_subject.as_str())
                                        .await?
                                        .try_collect()
                                        .await?;

                                    let message_count_on_subject =
                                        info.get(fut_subject.as_str()).copied().ok_or(
                                            Error::CalculateReadWindowCountNotFound(fut_subject),
                                        )?;

                                    Ok(message_count_on_subject)
                                }),
                            };
                            continue;
                        }
                        // Subscription stream yielded an error as the next item
                        Poll::Ready(Some(Err(err))) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // We can't delete this message easily as the sequence number
                            // comes from the [`Info`] struct, so we're going to restart
                            // the whole process
                            trace!(
                                si.error.message = ?err,
                                "error on subscription stream on first read; skipping message",
                            );

                            // Set next state and return error
                            *this.state = State::ReadFirstMessage;
                            span.record("task.state", this.state.as_ref());
                            let err = span.record_err(err);
                            drop(guard);
                            *this.span = None;
                            return Poll::Ready(Some(Err(Error::ReadFirstMessage(Box::new(err)))));
                        }
                        // Subscription stream has closed, so we close
                        Poll::Ready(None) => {
                            span.record("task.state", this.state.as_ref());
                            span.record_ok();
                            return Poll::Ready(None);
                        }
                        // Pending on the first message, so we are pending too
                        Poll::Pending => {
                            let pending_duration = poll_start.elapsed();
                            if pending_duration.as_millis() > 0 {
                                debug!(
                                    pending_duration_ms = pending_duration.as_millis(),
                                    "subscription.poll_next_unpin returned Pending for first message",
                                );
                            }
                            return Poll::Pending;
                        }
                    }
                }
                // 2. Calculating the number of messages to read, a.k.a the "read window"
                State::CalculateReadWindow {
                    subject,
                    message,
                    message_stream_sequence,
                    calculate_read_window_fut,
                } => {
                    // Caclulate the number of messages available to read, a.k.a the "read window".
                    // This is the number of messages we will unconditionally read in as our "read
                    // window".
                    let calc_start = Instant::now();
                    match calculate_read_window_fut.poll_unpin(cx) {
                        // Read window calculated successfully
                        Poll::Ready(Ok(read_window)) => {
                            let calc_duration = calc_start.elapsed();
                            debug!(
                                calc_duration_ms = calc_duration.as_millis(),
                                read_window = read_window,
                                "calculate_read_window completed",
                            );

                            // Metrics: Track read window calculation latency and size
                            histogram!(
                                compressing_stream_read_window_calc_latency_ms =
                                    calc_duration.as_millis() as f64
                            );
                            histogram!(compressing_stream_read_window_size = read_window as f64);

                            update_heartbeat(this.last_compressing_heartbeat_tx);
                            span.record("read_window.count", read_window);

                            let message = message
                                .take()
                                .expect("extracting owned value only happens once");

                            debug!(
                                first_message_info_pending = message
                                    .info()
                                    .map(|info| info.pending as isize)
                                    .unwrap_or(-1),
                                stream_info_with_subjects_count = read_window,
                                "calculated number of messages to read"
                            );

                            // Set next state and continue loop
                            *this.state = State::ParseFirstRequest {
                                subject: subject.take(),
                                read_window,
                                message_stream_sequence: *message_stream_sequence,
                                parse_message_fut: Box::pin(async move { parse_message(message) }),
                            };
                            continue;
                        }
                        // Failed to determine read window
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // We can't determine the read window, so we'll set it to `1` and
                            // continue as there is work we can do with the first message
                            warn!(
                                si.error.message = ?err,
                                concat!(
                                    "failed calculate read window; ",
                                    "setting value to `1`",
                                ),
                            );

                            let read_window = 1;

                            let message = message
                                .take()
                                .expect("extracting owned value only happens once");

                            // Set next state and continue loop
                            *this.state = State::ParseFirstRequest {
                                subject: subject.take(),
                                read_window,
                                message_stream_sequence: *message_stream_sequence,
                                parse_message_fut: Box::pin(async move { parse_message(message) }),
                            };
                            continue;
                        }
                        // Pending on parse message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 3. Parsing the first message into an API request
                State::ParseFirstRequest {
                    subject,
                    read_window,
                    message_stream_sequence,
                    parse_message_fut,
                } => {
                    // Parse API request from Jetstream message
                    match parse_message_fut.poll_unpin(cx) {
                        // API request parsed successfully
                        Poll::Ready(Ok((request, headers))) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            let mut requests = Vec::with_capacity(*read_window);
                            requests.push(request);

                            let mut stream_sequence_numbers = VecDeque::with_capacity(*read_window);
                            stream_sequence_numbers.push_back(*message_stream_sequence);

                            // Extract span context from headers for span linking. This is the
                            // first request, so we need to initialize the contexts vec collection.
                            let mut span_contexts = Vec::with_capacity(*read_window);
                            if let Some(ref headers) = headers {
                                let otel_ctx = propagation::extract_opentelemetry_context(headers);
                                span_contexts.push(otel_ctx.span().span_context().clone());
                                span.set_parent(otel_ctx);
                            }

                            // We've read all message in the read window
                            if requests.len() == *read_window {
                                // There are no requests to compress
                                if requests.is_empty() {
                                    // Pop the first sequence number off the delete list
                                    match stream_sequence_numbers.pop_front() {
                                        // A message was popped off list
                                        Some(message_stream_sequence) => {
                                            let stream = this.stream.clone();
                                            // Set next state and continue loop
                                            *this.state = State::DeleteStreamMessage {
                                                subject: subject.take(),
                                                delete_message_fut: Box::pin(async move {
                                                    stream
                                                        .delete_message(message_stream_sequence)
                                                        .await
                                                        .map(|_| ())
                                                }),
                                                compressed_request: Some(None),
                                                stream_sequence_numbers,
                                                deleted_count: 0,
                                            };
                                            continue;
                                        }
                                        // The delete list is empty
                                        None => {
                                            // Nothing to compress and nothing to delete, so
                                            // re-start state machine

                                            span.record("messages.deleted.count", 0);

                                            // Set next state and continue loop
                                            *this.state = State::ReadFirstMessage;
                                            drop(guard);
                                            *this.span = None;
                                            continue;
                                        }
                                    }
                                }
                                // There are requests to compress
                                else {
                                    span.record("requests.count", requests.len());

                                    // Set next state and continue loop
                                    *this.state = State::CompressRequests {
                                        subject: subject.take(),
                                        compress_messages_fut: Box::pin(async move {
                                            C::compress_from_requests(requests).await
                                        }),
                                        stream_sequence_numbers,
                                        span_contexts,
                                    };
                                    continue;
                                }
                            }
                            // There are remaining messages in the read window
                            else {
                                // Set next state and continue loop
                                *this.state = State::ReadNextMessageInWindow {
                                    subject: subject.take(),
                                    read_window: *read_window,
                                    requests,
                                    stream_sequence_numbers,
                                    span_contexts,
                                };
                                continue;
                            }
                        }
                        // Failed to parse API request from message
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Set next state to delete this message and restart the state
                            trace!(
                                si.error.message = ?err,
                                "failed to parse api request from first message; deleting message",
                            );

                            let stream = this.stream.clone();
                            let message_stream_sequence = *message_stream_sequence;

                            // Set next state and return error
                            *this.state = State::DeleteFirstMessageAfterError {
                                subject: subject.take(),
                                delete_message_fut: Box::pin(async move {
                                    stream
                                        .delete_message(message_stream_sequence)
                                        .await
                                        .map(|_| ())
                                }),
                            };
                            span.record("task.state", this.state.as_ref());
                            let err = span.record_err(err);
                            return Poll::Ready(Some(Err(Error::ParseFirstRequest(err))));
                        }
                        // Pending on parse message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 4. Reading the next message from the subscription in the read window
                State::ReadNextMessageInWindow {
                    subject,
                    read_window,
                    requests,
                    stream_sequence_numbers,
                    span_contexts,
                } => {
                    // Read next message from subscription in read window
                    let poll_start = Instant::now();
                    match this.subscription.poll_next_unpin(cx) {
                        Poll::Ready(Some(Ok(message))) => {
                            let poll_duration = poll_start.elapsed();
                            debug!(
                                poll_duration_ms = poll_duration.as_millis(),
                                requests_so_far = requests.len(),
                                read_window = *read_window,
                                "subscription.poll_next_unpin completed for next message in window",
                            );

                            // Metrics: Track subscription poll latency and message count
                            histogram!(
                                compressing_stream_subscription_poll_latency_ms =
                                    poll_duration.as_millis() as f64,
                                message_position = "subsequent"
                            );
                            monotonic!(
                                compressing_stream_messages_read = 1,
                                message_position = "subsequent"
                            );

                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Determine the stream sequence number of this message so we can
                            // delete it later.
                            let message_stream_sequence = match message.info() {
                                // Info parsed successfully from the message
                                Ok(info) => info.stream_sequence,
                                // Failed to parse [`Info`] from message
                                Err(err) => {
                                    // We can't delete this message easily as the sequence number
                                    // comes from the [`Info`] struct, so we're going to move to
                                    // compress what has been accumulated
                                    trace!(
                                        si.error.message = ?err,
                                        messaging.destination.name = message.subject.as_str(),
                                        concat!(
                                            "failed to parse info from next message; ",
                                            "skipping message & compressing remaining",
                                        ),
                                    );

                                    let requests = mem::take(requests);

                                    span.record("requests.count", requests.len());

                                    // Set next state and return error
                                    *this.state = State::CompressRequests {
                                        subject: subject.take(),
                                        compress_messages_fut: Box::pin(async move {
                                            C::compress_from_requests(requests).await
                                        }),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        span_contexts: mem::take(span_contexts),
                                    };
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(Error::NextMessageInfoParse(
                                        err,
                                    ))));
                                }
                            };

                            // Set next state and continue loop
                            *this.state = State::ParseNextRequestInWindow {
                                subject: subject.take(),
                                read_window: *read_window,
                                message_stream_sequence,
                                parse_message_fut: Box::pin(async move { parse_message(message) }),
                                requests: mem::take(requests),
                                stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                span_contexts: mem::take(span_contexts),
                            };
                            continue;
                        }
                        // Subscription stream yielded an error as the next item
                        Poll::Ready(Some(Err(err))) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Set up state to move into compression of what has been accumulated
                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "error on subscription stream on next read; ",
                                    "skipping message & compressing remaining",
                                ),
                            );

                            let requests = mem::take(requests);

                            span.record("requests.count", requests.len());

                            // Set next state and return error
                            *this.state = State::CompressRequests {
                                subject: subject.take(),
                                compress_messages_fut: Box::pin(async move {
                                    C::compress_from_requests(requests).await
                                }),
                                stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                span_contexts: mem::take(span_contexts),
                            };
                            span.record("task.state", this.state.as_ref());
                            let err = span.record_err(err);
                            return Poll::Ready(Some(Err(Error::ReadNextMessageInWindow(
                                Box::new(err),
                            ))));
                        }
                        // Subscription stream has closed
                        Poll::Ready(None) => {
                            // Set up state to compress remaining and then close our stream
                            trace!(concat!(
                                "subscription stream is closed on next read; ",
                                "compressing remaining & closing stream",
                            ));

                            let requests = mem::take(requests);

                            // Set next state and continue loop
                            *this.state = State::CompressRequestsAndClose {
                                subject: subject.take(),
                                compress_messages_fut: Box::pin(async move {
                                    C::compress_from_requests(requests).await
                                }),
                                stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                span_contexts: mem::take(span_contexts),
                            };
                            continue;
                        }
                        // Pending on the next message, so we are pending too
                        Poll::Pending => {
                            let pending_duration = poll_start.elapsed();
                            if pending_duration.as_millis() > 0 {
                                debug!(
                                    pending_duration_ms = pending_duration.as_millis(),
                                    requests_so_far = requests.len(),
                                    read_window = *read_window,
                                    "subscription.poll_next_unpin returned Pending for next message in window",
                                );
                            }
                            return Poll::Pending;
                        }
                    }
                }
                // 5. Parsing the next message into an API request in the read window
                State::ParseNextRequestInWindow {
                    subject,
                    read_window,
                    message_stream_sequence,
                    parse_message_fut,
                    requests,
                    stream_sequence_numbers,
                    span_contexts,
                } => {
                    // Parse API request from Jetstream message
                    match parse_message_fut.poll_unpin(cx) {
                        // API request parsed successfully
                        Poll::Ready(Ok((request, headers))) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            requests.push(request);
                            stream_sequence_numbers.push_back(*message_stream_sequence);

                            // Extract span context from headers for span linking. We already have
                            // a span context from the first message, so now we'll add another.
                            if let Some(ref headers) = headers {
                                let otel_ctx = propagation::extract_opentelemetry_context(headers);
                                let span_ctx = otel_ctx.span().span_context().clone();
                                span_contexts.push(span_ctx);
                            }

                            let requests = mem::take(requests);
                            let stream_sequence_numbers = mem::take(stream_sequence_numbers);
                            let span_contexts = mem::take(span_contexts);

                            // We've read all message in the read window
                            if requests.len() == *read_window {
                                span.record("requests.count", requests.len());

                                // Set next state and continue loop
                                *this.state = State::CompressRequests {
                                    subject: subject.take(),
                                    compress_messages_fut: Box::pin(async move {
                                        C::compress_from_requests(requests).await
                                    }),
                                    stream_sequence_numbers,
                                    span_contexts,
                                };
                                continue;
                            }
                            // There are remaining messages in the read window
                            else {
                                // Set next state and continue loop
                                *this.state = State::ReadNextMessageInWindow {
                                    subject: subject.take(),
                                    read_window: *read_window,
                                    requests,
                                    stream_sequence_numbers,
                                    span_contexts,
                                };
                                continue;
                            }
                        }
                        // Failed to parse API request from message
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Set next state to compress remaining requests
                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "failed to parse api request from next message; ",
                                    "skipping message & compressing remaining",
                                ),
                            );

                            // Add current message to list of messages to delete
                            stream_sequence_numbers.push_back(*message_stream_sequence);

                            let requests = mem::take(requests);
                            let stream_sequence_numbers = mem::take(stream_sequence_numbers);
                            let span_contexts = mem::take(span_contexts);

                            span.record("requests.count", requests.len());

                            // Set next state and return error
                            *this.state = State::CompressRequests {
                                subject: subject.take(),
                                compress_messages_fut: Box::pin(async move {
                                    C::compress_from_requests(requests).await
                                }),
                                stream_sequence_numbers,
                                span_contexts,
                            };
                            span.record("task.state", this.state.as_ref());
                            let err = span.record_err(err);
                            return Poll::Ready(Some(Err(Error::ParseNextRequestInWindow(err))));
                        }
                        // Pending on parse message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 6. Compressing multiple API requests into a single compressed request
                State::CompressRequests {
                    subject,
                    compress_messages_fut,
                    stream_sequence_numbers,
                    span_contexts,
                } => {
                    // Add span links to track the lineage from the original incoming requests to
                    // the compressed request. Links must be added before any other work is done
                    // within the span.
                    for span_context in span_contexts.iter() {
                        if span_context.is_valid() {
                            span.add_link(span_context.clone());
                        } else {
                            monotonic!(
                                compressing_stream_span_context_invalid_for_linking = 1,
                                should_close = "yes"
                            );
                        }
                    }

                    // Compress multiple API requests into a single compressed request
                    let compress_start = Instant::now();
                    match compress_messages_fut.poll_unpin(cx) {
                        // Requests compressed successfully
                        Poll::Ready(Ok(compressed_request)) => {
                            let compress_duration = compress_start.elapsed();

                            // Metrics: Track compression operation
                            histogram!(
                                compressing_stream_compress_latency_ms =
                                    compress_duration.as_millis() as f64
                            );
                            monotonic!(
                                compressing_stream_compress_operations = 1,
                                result = "success"
                            );

                            update_heartbeat(this.last_compressing_heartbeat_tx);
                            span.record("compressed.kind", compressed_request.as_ref());

                            // Pop the first sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();
                                    // Set next state and continue loop
                                    *this.state = State::DeleteStreamMessage {
                                        subject: subject.take(),
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        compressed_request: Some(Some(compressed_request)),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: 0,
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", 0);

                                    // Set next state and continue loop
                                    *this.state = State::YieldItem {
                                        subject: subject.take(),
                                        compressed_request: Some(compressed_request),
                                    };
                                    continue;
                                }
                            }
                        }
                        // Error while compressing requests
                        Poll::Ready(Err(err)) => {
                            // Metrics: Track compression failures
                            monotonic!(
                                compressing_stream_compress_operations = 1,
                                result = "error"
                            );

                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Nothing much we can do at this point, if we can't compress then we
                            // throw all the API requests away and delete the associated messages

                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "failed to compress requests; ",
                                    "skipping requests & deleting messages",
                                ),
                            );

                            // Pop the first sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off the delete list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and return error
                                    *this.state = State::DeleteStreamMessageAfterCompressError {
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: 0,
                                    };
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(Error::CompressedRequest(err))));
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", 0);

                                    // Nothing to compress and nothing to delete, so
                                    // re-start state machine

                                    // Set next state and return error
                                    *this.state = State::ReadFirstMessage;
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    drop(guard);
                                    *this.span = None;
                                    return Poll::Ready(Some(Err(Error::CompressedRequest(err))));
                                }
                            }
                        }
                        // Pending on compressing messages, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 7. Deleting a message from the Jetstream stream
                State::DeleteStreamMessage {
                    subject,
                    delete_message_fut,
                    compressed_request,
                    stream_sequence_numbers,
                    deleted_count,
                } => {
                    // Delete a message
                    let delete_start = Instant::now();
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
                            let delete_duration = delete_start.elapsed();

                            // Metrics: Track message deletion
                            histogram!(
                                compressing_stream_message_delete_latency_ms =
                                    delete_duration.as_millis() as f64
                            );
                            monotonic!(compressing_stream_messages_deleted = 1, result = "success");

                            update_heartbeat(this.last_compressing_heartbeat_tx);
                            *deleted_count += 1;

                            let compressed_request = compressed_request
                                .take()
                                .expect("extracting owned value only happens once");

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and continue loop
                                    *this.state = State::DeleteStreamMessage {
                                        subject: subject.take(),
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        compressed_request: Some(compressed_request),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: *deleted_count,
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Do we have a compressed request?
                                    match compressed_request {
                                        // Compressed request found
                                        Some(compressed_request) => {
                                            // Set next state and continue loop
                                            *this.state = State::YieldItem {
                                                subject: subject.take(),
                                                compressed_request: Some(compressed_request),
                                            };
                                            continue;
                                        }
                                        // No compressed request
                                        None => {
                                            // Nothing to yield so re-start state machine

                                            // Set next state and continue loop
                                            *this.state = State::ReadFirstMessage;
                                            drop(guard);
                                            *this.span = None;
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        // Error when deleting a message
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // This message failed to delete, so return the error but set state to
                            // continue the process of deleting the remaining messages
                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "failed to delete message from stream; ",
                                    "skipping message & deleting remaining",
                                ),
                            );

                            let compressed_request = compressed_request
                                .take()
                                .expect("extracting owned value only happens once");

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and return error
                                    *this.state = State::DeleteStreamMessage {
                                        subject: subject.take(),
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        compressed_request: Some(compressed_request),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: *deleted_count,
                                    };
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(Error::DeleteStreamMessage(err))));
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Do we have a compressed request?
                                    match compressed_request {
                                        // Compressed request found
                                        Some(compressed_request) => {
                                            // Set next state and return error
                                            *this.state = State::YieldItem {
                                                subject: subject.take(),
                                                compressed_request: Some(compressed_request),
                                            };
                                            span.record("task.state", this.state.as_ref());
                                            let err = span.record_err(err);
                                            return Poll::Ready(Some(Err(
                                                Error::DeleteStreamMessage(err),
                                            )));
                                        }
                                        // No compressed request
                                        None => {
                                            // Nothing to yield so re-start state machine

                                            // Set next state and return error
                                            *this.state = State::ReadFirstMessage;
                                            span.record("task.state", this.state.as_ref());
                                            let err = span.record_err(err);
                                            drop(guard);
                                            *this.span = None;
                                            return Poll::Ready(Some(Err(
                                                Error::DeleteStreamMessage(err),
                                            )));
                                        }
                                    }
                                }
                            }
                        }
                        // Pending on deleting message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 8. Converting request into final message to yield from [`Stream`]
                State::YieldItem {
                    subject,
                    compressed_request,
                } => {
                    update_heartbeat(this.last_compressing_heartbeat_tx);

                    // Metrics: Track batch span duration from start to yield
                    if let Some(span_start) = this.span_started_at.take() {
                        let batch_duration = span_start.elapsed();
                        histogram!(
                            compressing_stream_batch_duration_ms =
                                batch_duration.as_millis() as f64
                        );
                    }
                    monotonic!(compressing_stream_items_yielded = 1);

                    let subject = subject
                        .take()
                        .expect("extracting owned value only happens once");

                    let compressed_request = compressed_request
                        .take()
                        .expect("extracting owned value only happens once");

                    let payload = match serde_json::to_vec(&compressed_request) {
                        // Compressed requests serialized to bytes successfully
                        Ok(vec) => vec.into(),
                        // Failed to serialize
                        Err(err) => {
                            // It's too bad this error (which it shouldn't) because all we can do
                            // is throw away the compressed request and re-start the state machine
                            trace!(
                                si.error.message = ?err,
                                messaging.destination.name = subject.as_str(),
                                "failed to serialize compressed request to local message",
                            );

                            // Set next state and return error
                            *this.state = State::ReadFirstMessage;
                            span.record("task.state", this.state.as_ref());
                            let err = span.record_err(err);
                            drop(guard);
                            *this.span = None;
                            return Poll::Ready(Some(Err(Error::SerializeLocalMessage(err))));
                        }
                    };

                    // We need to inject headers with our span and all of its links that we've
                    // built. Without this, the trace would end here.
                    let mut headers = HeaderMap::new();
                    propagation::inject_headers(&mut headers);

                    let local_message = LocalMessage {
                        subject,
                        headers: Some(headers),
                        payload,
                    };

                    // Set next state and return item
                    *this.state = State::ReadFirstMessage;
                    span.record("task.state", this.state.as_ref());
                    span.record_ok();
                    drop(guard);
                    *this.span = None;
                    return Poll::Ready(Some(Ok(local_message)));
                }
                // 3.1 Deleting the initial message after error
                State::DeleteFirstMessageAfterError {
                    subject,
                    delete_message_fut,
                } => {
                    // Delete the message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);
                            span.record("messages.deleted.count", 1);
                        }
                        // Error when deleting message
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            let subject = subject
                                .take()
                                .expect("extracting owned value only happens once");
                            error!(
                                si.error.message = ?err,
                                messaging.destination.name = subject.as_str(),
                                "failed to delete message from stream",
                            );
                        }
                        // Pending on deleting message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    };

                    // Set next state and continue loop
                    *this.state = State::ReadFirstMessage;
                    drop(guard);
                    *this.span = None;
                    continue;
                }
                // 4.1 Compressing remaining requests before closing [`Stream`]
                State::CompressRequestsAndClose {
                    subject,
                    compress_messages_fut,
                    stream_sequence_numbers,
                    span_contexts,
                } => {
                    // Add span links to track the lineage from the original incoming requests to
                    // the compressed request. Links must be added before any other work is done
                    // within the span.
                    for span_context in span_contexts.iter() {
                        if span_context.is_valid() {
                            span.add_link(span_context.clone());
                        } else {
                            monotonic!(
                                compressing_stream_span_context_invalid_for_linking = 1,
                                should_close = "no"
                            );
                        }
                    }

                    // Compress multiple API requests into a single compressed request
                    match compress_messages_fut.poll_unpin(cx) {
                        // Requests compressed successfully
                        Poll::Ready(Ok(compressed_request)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Pop the first sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and continue loop
                                    *this.state = State::DeleteStreamMessageAndClose {
                                        subject: subject.take(),
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        compressed_request: Some(Some(compressed_request)),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: 0,
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", 0);

                                    // Set next state and continue loop
                                    *this.state = State::YieldItemAndClose {
                                        subject: subject.take(),
                                        compressed_request: Some(compressed_request),
                                    };
                                    continue;
                                }
                            }
                        }
                        // Error while compressing requests
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // Nothing much we can do at this point, if we can't compress then we
                            // throw all the API requests away and delete the associated messages

                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "failed to compress requests; ",
                                    "skipping requests, deleting messages & closing",
                                ),
                            );

                            // Pop the first sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off the delete list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and return error
                                    *this.state =
                                        State::DeleteStreamMessageAfterCompressErrorAndClose {
                                            delete_message_fut: Box::pin(async move {
                                                stream
                                                    .delete_message(message_stream_sequence)
                                                    .await
                                                    .map(|_| ())
                                            }),
                                            stream_sequence_numbers: mem::take(
                                                stream_sequence_numbers,
                                            ),
                                            deleted_count: 0,
                                        };
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(
                                        Error::CompressedRequestBeforeClose(err),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", 0);

                                    // Nothing to compress and nothing to delete, so
                                    // move to close stream

                                    // Set next state and return error
                                    *this.state = State::CloseStream;
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(
                                        Error::CompressedRequestBeforeClose(err),
                                    )));
                                }
                            }
                        }
                        // Pending on compressing messages, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 4.2 Deleting messages from the Jetstream stream before closing [`Stream`]
                State::DeleteStreamMessageAndClose {
                    subject,
                    delete_message_fut,
                    compressed_request,
                    stream_sequence_numbers,
                    deleted_count,
                } => {
                    // Delete a message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);
                            *deleted_count += 1;

                            let compressed_request = compressed_request
                                .take()
                                .expect("extracting owned value only happens once");

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and continue loop
                                    *this.state = State::DeleteStreamMessageAndClose {
                                        subject: subject.take(),
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        compressed_request: Some(compressed_request),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: *deleted_count,
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Do we have a compressed request?
                                    match compressed_request {
                                        // Compressed request found
                                        Some(compressed_request) => {
                                            // Set next state and continue loop
                                            *this.state = State::YieldItemAndClose {
                                                subject: subject.take(),
                                                compressed_request: Some(compressed_request),
                                            };
                                            continue;
                                        }
                                        // No compressed request
                                        None => {
                                            // Nothing to yield so move to close stream

                                            // Set next state and continue loop
                                            *this.state = State::CloseStream;
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        // Error when deleting a message
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // This message failed to delete, so return the error but set state to
                            // continue the process of deleting the remaining messages
                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "failed to delete message from stream; ",
                                    "skipping message, deleting remaining & closing",
                                ),
                            );

                            let compressed_request = compressed_request
                                .take()
                                .expect("extracting owned value only happens once");

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and return error
                                    *this.state = State::DeleteStreamMessage {
                                        subject: subject.take(),
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        compressed_request: Some(compressed_request),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: *deleted_count,
                                    };
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageBeforeClose(err),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Do we have a compressed request?
                                    match compressed_request {
                                        // Compressed request found
                                        Some(compressed_request) => {
                                            // Set next state and return error
                                            *this.state = State::YieldItemAndClose {
                                                subject: subject.take(),
                                                compressed_request: Some(compressed_request),
                                            };
                                            span.record("task.state", this.state.as_ref());
                                            let err = span.record_err(err);
                                            return Poll::Ready(Some(Err(
                                                Error::DeleteStreamMessageBeforeClose(err),
                                            )));
                                        }
                                        // No compressed request
                                        None => {
                                            // Nothing to yield so re-start state machine

                                            // Set next state and return error
                                            *this.state = State::CloseStream;
                                            span.record("task.state", this.state.as_ref());
                                            let err = span.record_err(err);
                                            return Poll::Ready(Some(Err(
                                                Error::DeleteStreamMessageBeforeClose(err),
                                            )));
                                        }
                                    }
                                }
                            }
                        }
                        // Pending on deleting message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 4.3. Converting request into final message to yield from [`Stream`] before
                // closing [`Stream`]
                State::YieldItemAndClose {
                    subject,
                    compressed_request,
                } => {
                    update_heartbeat(this.last_compressing_heartbeat_tx);

                    let subject = subject
                        .take()
                        .expect("extracting owned value only happens once");

                    let compressed_request = compressed_request
                        .take()
                        .expect("extracting owned value only happens once");

                    let payload = match serde_json::to_vec(&compressed_request) {
                        // Compressed requests serialized to bytes successfully
                        Ok(vec) => vec.into(),
                        // Failed to serialize
                        Err(err) => {
                            // It's too bad this error (which it shouldn't) because all we can do
                            // is throw away the compressed request and close the stream
                            trace!(
                                si.error.message = ?err,
                                messaging.destination.name = subject.as_str(),
                                concat!(
                                    "failed to serialize compressed request to local message; ",
                                    "closing",
                                ),
                            );

                            // Set next state and return error
                            *this.state = State::CloseStream;
                            span.record("task.state", this.state.as_ref());
                            let err = span.record_err(err);
                            return Poll::Ready(Some(Err(
                                Error::SerializeLocalMessageBeforeClose(err),
                            )));
                        }
                    };

                    // We need to inject headers with our span and all of its links that we've
                    // built. Without this, the trace would end here.
                    let mut headers = HeaderMap::new();
                    propagation::inject_headers(&mut headers);

                    let local_message = LocalMessage {
                        subject,
                        headers: Some(headers),
                        payload,
                    };

                    // Set next state and return item
                    *this.state = State::CloseStream;
                    span.record("task.state", this.state.as_ref());
                    span.record_ok();
                    return Poll::Ready(Some(Ok(local_message)));
                }
                // 4.1.1 Closing the stream
                State::CloseStream => {
                    span.record("task.state", this.state.as_ref());
                    // Don't record span either way as it may have already been marked ok/err
                    return Poll::Ready(None);
                }
                // 6.1 Deleting messages from the Jetstream stream after failing to compress
                // requests
                State::DeleteStreamMessageAfterCompressError {
                    delete_message_fut,
                    stream_sequence_numbers,
                    deleted_count,
                } => {
                    // Delete a message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);
                            *deleted_count += 1;

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and continue loop
                                    *this.state = State::DeleteStreamMessageAfterCompressError {
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: *deleted_count,
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Nothing to yield so re-start state machine

                                    // Set next state and continue loop
                                    *this.state = State::ReadFirstMessage;
                                    drop(guard);
                                    *this.span = None;
                                    continue;
                                }
                            }
                        }
                        // Error when deleting a message
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // This message failed to delete, so return the error but set state to
                            // continue the process of deleting the remaining messages
                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "failed to delete message from stream; ",
                                    "skipping message & deleting remaining",
                                ),
                            );

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and return error
                                    *this.state = State::DeleteStreamMessageAfterCompressError {
                                        delete_message_fut: Box::pin(async move {
                                            stream
                                                .delete_message(message_stream_sequence)
                                                .await
                                                .map(|_| ())
                                        }),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                        deleted_count: *deleted_count,
                                    };
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageAfterCompressError(err),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Nothing to yield so re-start state machine

                                    // Set next state and return error
                                    *this.state = State::ReadFirstMessage;
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    drop(guard);
                                    *this.span = None;
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageAfterCompressError(err),
                                    )));
                                }
                            }
                        }
                        // Pending on deleting message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 4.1.2 Deleting messages from the Jetstream stream after failing to compress
                // requests before closing [`Stream`]
                State::DeleteStreamMessageAfterCompressErrorAndClose {
                    delete_message_fut,
                    stream_sequence_numbers,
                    deleted_count,
                } => {
                    // Delete a message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);
                            *deleted_count += 1;

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and continue loop
                                    *this.state =
                                        State::DeleteStreamMessageAfterCompressErrorAndClose {
                                            delete_message_fut: Box::pin(async move {
                                                stream
                                                    .delete_message(message_stream_sequence)
                                                    .await
                                                    .map(|_| ())
                                            }),
                                            stream_sequence_numbers: mem::take(
                                                stream_sequence_numbers,
                                            ),
                                            deleted_count: *deleted_count,
                                        };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Nothing to yield so re-start state machine

                                    // Set next state and continue loop
                                    *this.state = State::CloseStream;
                                    continue;
                                }
                            }
                        }
                        // Error when deleting a message
                        Poll::Ready(Err(err)) => {
                            update_heartbeat(this.last_compressing_heartbeat_tx);

                            // This message failed to delete, so return the error but set state to
                            // continue the process of deleting the remaining messages
                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "failed to delete message from stream; ",
                                    "skipping message, deleting remaining & closing",
                                ),
                            );

                            // Pop the next sequence number off the delete list
                            match stream_sequence_numbers.pop_front() {
                                // A message was popped off list
                                Some(message_stream_sequence) => {
                                    let stream = this.stream.clone();

                                    // Set next state and return error
                                    *this.state =
                                        State::DeleteStreamMessageAfterCompressErrorAndClose {
                                            delete_message_fut: Box::pin(async move {
                                                stream
                                                    .delete_message(message_stream_sequence)
                                                    .await
                                                    .map(|_| ())
                                            }),
                                            stream_sequence_numbers: mem::take(
                                                stream_sequence_numbers,
                                            ),
                                            deleted_count: *deleted_count,
                                        };
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageAfterCompressErrorBeforeClose(
                                            err,
                                        ),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    span.record("messages.deleted.count", deleted_count);

                                    // Nothing to yield so close stream

                                    // Set next state and return error
                                    *this.state = State::CloseStream;
                                    span.record("task.state", this.state.as_ref());
                                    let err = span.record_err(err);
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageAfterCompressErrorBeforeClose(
                                            err,
                                        ),
                                    )));
                                }
                            }
                        }
                        // Pending on deleting message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}

#[inline]
fn update_heartbeat(heartbeat_tx: &mut Option<watch::Sender<Instant>>) {
    if let Some(heartbeat_tx) = heartbeat_tx {
        // Update the "liveness" of the stream to prevent a quiescent period if there is
        // still work to do
        heartbeat_tx.send_replace(Instant::now());
    }
}

#[inline]
fn parse_message<R>(
    message: jetstream::Message,
) -> result::Result<(R, Option<HeaderMap>), NegotiateError>
where
    R: Negotiate + Send + 'static,
{
    let (head, payload) = message.into_head_and_payload();
    let content_info =
        ContentInfo::try_from(head.headers.as_ref()).map_err(ContentInfoError::from_err)?;

    let request = R::negotiate(&content_info, &payload)?;
    Ok((request, head.headers))
}
