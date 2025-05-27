use std::{
    collections::VecDeque,
    error,
    fmt,
    mem,
    pin::Pin,
    result,
    task::{
        Context,
        Poll,
    },
};

use futures::{
    FutureExt,
    Stream,
    StreamExt,
    future::BoxFuture,
};
use naxum::extract::FromMessage;
use pin_project_lite::pin_project;
use si_data_nats::{
    Subject,
    async_nats::jetstream::{
        self,
        stream::DeleteMessageError,
    },
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    compressed_request::{
        CompressedRequest,
        CompressedRequestError,
    },
    extract::{
        ApiTypesNegotiate,
        ApiTypesNegotiateRejection,
        EddaRequestKind,
    },
    local_message::LocalMessage,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CompressingStreamError {
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
    ParseFirstRequest(ApiTypesNegotiateRejection),
    #[error(
        "failed to parse api request from next message; skipping message & compressing remaining: {0}"
    )]
    ParseNextRequestInWindow(ApiTypesNegotiateRejection),
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
#[derive(Default)]
enum State {
    #[default]
    /// 1. Reading the first message from the subscription
    ReadFirstMessage,
    /// 2. Parsing the first message into an API request
    ParseFirstRequest {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The number of messages to read
        read_window: usize,
        /// The stream sequence number of the first message
        message_stream_sequence: u64,
        /// A [`Future`] that parses a Jetstream message into an API request
        parse_message_fut:
            BoxFuture<'static, result::Result<ApiTypesNegotiate, ApiTypesNegotiateRejection>>,
    },
    /// 3. Reading the next message from the subscription in the read window
    ReadNextMessageInWindow {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The number of messages to read
        read_window: usize,
        /// The accumulated list of read and parsed API requests
        requests: Vec<EddaRequestKind>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
    },
    /// 4. Parsing the next message into an API request in the read window
    ParseNextRequestInWindow {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The number of messages to read
        read_window: usize,
        /// The stream sequence number of the initial message
        message_stream_sequence: u64,
        /// A [`Future`] that parses a Jetstream message into an API request
        parse_message_fut:
            BoxFuture<'static, result::Result<ApiTypesNegotiate, ApiTypesNegotiateRejection>>,
        /// The accumulated list of read and parsed API requests
        requests: Vec<EddaRequestKind>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
    },
    /// 5. Compressing multiple API requests into a single compressed request
    CompressRequests {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that compresses multiple API requests into a single "compressed" request
        compress_messages_fut:
            BoxFuture<'static, result::Result<CompressedRequest, CompressedRequestError>>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
    },
    /// 6. Deleting a message from the Jetstream stream
    DeleteStreamMessage {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
        /// The compressed request to later yield (outer [`Option`] for `mem::take()`, and inner is
        /// when there were no requests to be compressed)
        compressed_request: Option<Option<CompressedRequest>>,
    },
    /// 7. Converting request into final message to yield from [`Stream`]
    YieldItem {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The compressed request to later yield ([`Option`] for `mem::take()`)
        compressed_request: Option<CompressedRequest>,
    },
    /// 2.1 Deleting the first message after error
    DeleteFirstMessageAfterError {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
    },
    /// 3.1 Compressing remaining requests before closing [`Stream`]
    CompressRequestsAndClose {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that compresses multiple API requests into a single "compressed" request
        compress_messages_fut:
            BoxFuture<'static, result::Result<CompressedRequest, CompressedRequestError>>,
        /// The accumulated list of stream sequence numbers to later delete
        stream_sequence_numbers: VecDeque<u64>,
    },
    /// 3.2 Deleting messages from the Jetstream stream before closing [`Stream`]
    DeleteStreamMessageAndClose {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The compressed request to later yield (outer [`Option`] for `mem::take()`, and inner is
        /// when there were no requests to be compressed)
        compressed_request: Option<Option<CompressedRequest>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
    },
    /// 3.3. Converting request into final message to yield from [`Stream`] before closing
    /// [`Stream`]
    YieldItemAndClose {
        /// The [`Subject`] to be used on the compressed request ([`Option`] for `mem::take()`)
        subject: Option<Subject>,
        /// The compressed request to later yield ([`Option`] for `mem::take()`)
        compressed_request: Option<CompressedRequest>,
    },
    /// 3.1.1 Closing the stream
    CloseStream,
    /// 5.1 Deleting messages from the Jetstream stream after failing to compress requests
    DeleteStreamMessageAfterCompressError {
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
    },
    /// 3.1.2 Deleting messages from the Jetstream stream after failing to compress requests before
    /// closing [`Stream`]
    DeleteStreamMessageAfterCompressErrorAndClose {
        /// A [`Future`] that deletes a message from the Jetstream stream
        delete_message_fut: BoxFuture<'static, result::Result<(), DeleteMessageError>>,
        /// The remaining list of stream sequence numbers to delete
        stream_sequence_numbers: VecDeque<u64>,
    },
}

pin_project! {
    pub struct CompressingStream<S> {
        #[pin]
        subscription: S,
        stream: jetstream::stream::Stream,
        state: State,
    }
}

impl<S> CompressingStream<S> {
    /// Creates and return a new CompressingStream.
    pub fn new(subscription: S, stream: jetstream::stream::Stream) -> Self {
        Self {
            subscription,
            stream,
            state: Default::default(),
        }
    }
}

impl<S> fmt::Debug for CompressingStream<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressingStream").finish_non_exhaustive()
    }
}

impl<S, E> Stream for CompressingStream<S>
where
    S: Stream<Item = result::Result<jetstream::Message, E>>,
    E: error::Error + Send + Sync + 'static,
{
    type Item = Result<LocalMessage>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match this.state {
                // 1. Reading the first message from the subscription
                State::ReadFirstMessage => {
                    // Read first message from subscription
                    match this.subscription.poll_next_unpin(cx) {
                        // Read the first Jetstream message successfully
                        Poll::Ready(Some(Ok(message))) => {
                            // Determine the number of messages pending for this consumer. This is
                            // the number of messages we will unconditionally read in as our "read
                            // window".
                            //
                            // Determine the stream sequence number of this message so we can
                            // delete it later.
                            let (read_window, message_stream_sequence) = match message.info() {
                                // Info parsed successfully from the message
                                Ok(info) => (info.pending + 1, info.stream_sequence),
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
                                    return Poll::Ready(Some(Err(Error::FirstMessageInfoParse(
                                        err,
                                    ))));
                                }
                            };

                            let subject = Some(message.subject.clone());

                            // Set next state and continue loop
                            *this.state = State::ParseFirstRequest {
                                subject,
                                read_window: read_window as usize,
                                message_stream_sequence,
                                parse_message_fut: Box::pin(async move {
                                    ApiTypesNegotiate::from_message(message.into(), &()).await
                                }),
                            };
                            continue;
                        }
                        // Subscription stream yielded an error as the next item
                        Poll::Ready(Some(Err(err))) => {
                            // We can't delete this message easily as the sequence number
                            // comes from the [`Info`] struct, so we're going to restart
                            // the whole process
                            trace!(
                                si.error.message = ?err,
                                "error on subscription stream on first read; skipping message",
                            );

                            // Set next state and return error
                            *this.state = State::ReadFirstMessage;
                            return Poll::Ready(Some(Err(Error::ReadFirstMessage(Box::new(err)))));
                        }
                        // Subscription stream has closed, so we close
                        Poll::Ready(None) => return Poll::Ready(None),
                        // Pending on the first message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 2. Parsing the first message into an API request
                State::ParseFirstRequest {
                    subject,
                    read_window,
                    message_stream_sequence,
                    parse_message_fut,
                } => {
                    // Parse API request from Jetstream message
                    match parse_message_fut.poll_unpin(cx) {
                        // API request parsed successfully
                        Poll::Ready(Ok(ApiTypesNegotiate(request))) => {
                            let mut requests = Vec::with_capacity(*read_window);
                            requests.push(request);

                            let mut stream_sequence_numbers = VecDeque::with_capacity(*read_window);
                            stream_sequence_numbers.push_back(*message_stream_sequence);

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
                                            };
                                            continue;
                                        }
                                        // The delete list is empty
                                        None => {
                                            // Nothing to compress and nothing to delete, so
                                            // re-start state machine

                                            // Set next state and continue loop
                                            *this.state = State::ReadFirstMessage;
                                            continue;
                                        }
                                    }
                                }
                                // There are requests to compress
                                else {
                                    // Set next state and continue loop
                                    *this.state = State::CompressRequests {
                                        subject: subject.take(),
                                        compress_messages_fut: Box::pin(async move {
                                            CompressedRequest::from_requests(requests).await
                                        }),
                                        stream_sequence_numbers,
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
                                };
                                continue;
                            }
                        }
                        // Failed to parse API request from message
                        Poll::Ready(Err(rejection)) => {
                            // Set next state to delete this message and restart the state
                            trace!(
                                si.error.message = ?rejection,
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
                            return Poll::Ready(Some(Err(Error::ParseFirstRequest(rejection))));
                        }
                        // Pending on parse message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 3. Reading the next message from the subscription in the read window
                State::ReadNextMessageInWindow {
                    subject,
                    read_window,
                    requests,
                    stream_sequence_numbers,
                } => {
                    // Read next message from subscription in read window
                    match this.subscription.poll_next_unpin(cx) {
                        Poll::Ready(Some(Ok(message))) => {
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

                                    // Set next state and return error
                                    *this.state = State::CompressRequests {
                                        subject: subject.take(),
                                        compress_messages_fut: Box::pin(async move {
                                            CompressedRequest::from_requests(requests).await
                                        }),
                                        stream_sequence_numbers: mem::take(stream_sequence_numbers),
                                    };
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
                                parse_message_fut: Box::pin(async move {
                                    ApiTypesNegotiate::from_message(message.into(), &()).await
                                }),
                                requests: mem::take(requests),
                                stream_sequence_numbers: mem::take(stream_sequence_numbers),
                            };
                            continue;
                        }
                        // Subscription stream yielded an error as the next item
                        Poll::Ready(Some(Err(err))) => {
                            // Set up state to move into compression of what has been accumulated
                            trace!(
                                si.error.message = ?err,
                                concat!(
                                    "error on subscription stream on next read; ",
                                    "skipping message & compressing remaining",
                                ),
                            );

                            let requests = mem::take(requests);

                            // Set next state and return error
                            *this.state = State::CompressRequests {
                                subject: subject.take(),
                                compress_messages_fut: Box::pin(async move {
                                    CompressedRequest::from_requests(requests).await
                                }),
                                stream_sequence_numbers: mem::take(stream_sequence_numbers),
                            };
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
                                    CompressedRequest::from_requests(requests).await
                                }),
                                stream_sequence_numbers: mem::take(stream_sequence_numbers),
                            };
                            continue;
                        }
                        // Pending on the next message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 4. Parsing the next message into an API request in the read window
                State::ParseNextRequestInWindow {
                    subject,
                    read_window,
                    message_stream_sequence,
                    parse_message_fut,
                    requests,
                    stream_sequence_numbers,
                } => {
                    // Parse API request from Jetstream message
                    match parse_message_fut.poll_unpin(cx) {
                        // API request parsed successfully
                        Poll::Ready(Ok(ApiTypesNegotiate(request))) => {
                            requests.push(request);
                            stream_sequence_numbers.push_back(*message_stream_sequence);

                            let requests = mem::take(requests);
                            let stream_sequence_numbers = mem::take(stream_sequence_numbers);

                            // We've read all message in the read window
                            if requests.len() == *read_window {
                                // Set next state and continue loop
                                *this.state = State::CompressRequests {
                                    subject: subject.take(),
                                    compress_messages_fut: Box::pin(async move {
                                        CompressedRequest::from_requests(requests).await
                                    }),
                                    stream_sequence_numbers,
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
                                };
                                continue;
                            }
                        }
                        // Failed to parse API request from message
                        Poll::Ready(Err(rejection)) => {
                            // Set next state to compress remaining requests
                            trace!(
                                si.error.message = ?rejection,
                                concat!(
                                    "failed to parse api request from next message; ",
                                    "skipping message & compressing remaining",
                                ),
                            );

                            // Add current message to list of messages to delete
                            stream_sequence_numbers.push_back(*message_stream_sequence);

                            let requests = mem::take(requests);
                            let stream_sequence_numbers = mem::take(stream_sequence_numbers);

                            // Set next state and return error
                            *this.state = State::CompressRequests {
                                subject: subject.take(),
                                compress_messages_fut: Box::pin(async move {
                                    CompressedRequest::from_requests(requests).await
                                }),
                                stream_sequence_numbers,
                            };
                            return Poll::Ready(Some(Err(Error::ParseNextRequestInWindow(
                                rejection,
                            ))));
                        }
                        // Pending on parse message, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 5. Compressing multiple API requests into a single compressed request
                State::CompressRequests {
                    subject,
                    compress_messages_fut,
                    stream_sequence_numbers,
                } => {
                    // Compress multiple API requests into a single compressed request
                    match compress_messages_fut.poll_unpin(cx) {
                        // Requests compressed successfully
                        Poll::Ready(Ok(compressed_request)) => {
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
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
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
                                    };
                                    return Poll::Ready(Some(Err(Error::CompressedRequest(err))));
                                }
                                // The delete list is empty
                                None => {
                                    // Nothing to compress and nothing to delete, so
                                    // re-start state machine

                                    // Set next state and return error
                                    *this.state = State::ReadFirstMessage;
                                    return Poll::Ready(Some(Err(Error::CompressedRequest(err))));
                                }
                            }
                        }
                        // Pending on compressing messages, so we are pending too
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 6. Deleting a message from the Jetstream stream
                State::DeleteStreamMessage {
                    subject,
                    delete_message_fut,
                    compressed_request,
                    stream_sequence_numbers,
                } => {
                    // Delete a message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
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
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
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
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        // Error when deleting a message
                        Poll::Ready(Err(err)) => {
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
                                    };
                                    return Poll::Ready(Some(Err(Error::DeleteStreamMessage(err))));
                                }
                                // The delete list is empty
                                None => {
                                    // Do we have a compressed request?
                                    match compressed_request {
                                        // Compressed request found
                                        Some(compressed_request) => {
                                            // Set next state and return error
                                            *this.state = State::YieldItem {
                                                subject: subject.take(),
                                                compressed_request: Some(compressed_request),
                                            };
                                            return Poll::Ready(Some(Err(
                                                Error::DeleteStreamMessage(err),
                                            )));
                                        }
                                        // No compressed request
                                        None => {
                                            // Nothing to yield so re-start state machine

                                            // Set next state and return error
                                            *this.state = State::ReadFirstMessage;
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
                // 7. Converting request into final message to yield from [`Stream`]
                State::YieldItem {
                    subject,
                    compressed_request,
                } => {
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
                            return Poll::Ready(Some(Err(Error::SerializeLocalMessage(err))));
                        }
                    };

                    let local_message = LocalMessage {
                        subject,
                        headers: None, // TODO(fnichol): propagation headers?
                        payload,
                    };

                    // Set next state and return item
                    *this.state = State::ReadFirstMessage;
                    return Poll::Ready(Some(Ok(local_message)));
                }
                // 2.1 Deleting the initial message after error
                State::DeleteFirstMessageAfterError {
                    subject,
                    delete_message_fut,
                } => {
                    // Delete the message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {}
                        // Error when deleting message
                        Poll::Ready(Err(err)) => {
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
                    continue;
                }
                // 3.1 Compressing remaining requests before closing [`Stream`]
                State::CompressRequestsAndClose {
                    subject,
                    compress_messages_fut,
                    stream_sequence_numbers,
                } => {
                    // Compress multiple API requests into a single compressed request
                    match compress_messages_fut.poll_unpin(cx) {
                        // Requests compressed successfully
                        Poll::Ready(Ok(compressed_request)) => {
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
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
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
                                        };
                                    return Poll::Ready(Some(Err(
                                        Error::CompressedRequestBeforeClose(err),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    // Nothing to compress and nothing to delete, so
                                    // move to close stream

                                    // Set next state and return error
                                    *this.state = State::CloseStream;
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
                // 3.2 Deleting messages from the Jetstream stream before closing [`Stream`]
                State::DeleteStreamMessageAndClose {
                    subject,
                    delete_message_fut,
                    compressed_request,
                    stream_sequence_numbers,
                } => {
                    // Delete a message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
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
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
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
                                    };
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageBeforeClose(err),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    // Do we have a compressed request?
                                    match compressed_request {
                                        // Compressed request found
                                        Some(compressed_request) => {
                                            // Set next state and return error
                                            *this.state = State::YieldItemAndClose {
                                                subject: subject.take(),
                                                compressed_request: Some(compressed_request),
                                            };
                                            return Poll::Ready(Some(Err(
                                                Error::DeleteStreamMessageBeforeClose(err),
                                            )));
                                        }
                                        // No compressed request
                                        None => {
                                            // Nothing to yield so re-start state machine

                                            // Set next state and return error
                                            *this.state = State::CloseStream;
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
                // 3.3. Converting request into final message to yield from [`Stream`] before
                // closing [`Stream`]
                State::YieldItemAndClose {
                    subject,
                    compressed_request,
                } => {
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
                            return Poll::Ready(Some(Err(
                                Error::SerializeLocalMessageBeforeClose(err),
                            )));
                        }
                    };

                    let local_message = LocalMessage {
                        subject,
                        headers: None, // TODO(fnichol): propagation headers?
                        payload,
                    };

                    // Set next state and return item
                    *this.state = State::CloseStream;
                    return Poll::Ready(Some(Ok(local_message)));
                }
                // 3.1.1 Closing the stream
                State::CloseStream => {
                    return Poll::Ready(None);
                }
                // 5.1 Deleting messages from the Jetstream stream after failing to compress
                // requests
                State::DeleteStreamMessageAfterCompressError {
                    delete_message_fut,
                    stream_sequence_numbers,
                } => {
                    // Delete a message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
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
                                    };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    // Nothing to yield so re-start state machine

                                    // Set next state and continue loop
                                    *this.state = State::ReadFirstMessage;
                                    continue;
                                }
                            }
                        }
                        // Error when deleting a message
                        Poll::Ready(Err(err)) => {
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
                                    };
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageAfterCompressError(err),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    // Nothing to yield so re-start state machine

                                    // Set next state and return error
                                    *this.state = State::ReadFirstMessage;
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
                State::DeleteStreamMessageAfterCompressErrorAndClose {
                    delete_message_fut,
                    stream_sequence_numbers,
                } => {
                    // Delete a message
                    match delete_message_fut.poll_unpin(cx) {
                        // Message was deleted successfully
                        Poll::Ready(Ok(_)) => {
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
                                        };
                                    continue;
                                }
                                // The delete list is empty
                                None => {
                                    // Nothing to yield so re-start state machine

                                    // Set next state and continue loop
                                    *this.state = State::CloseStream;
                                    continue;
                                }
                            }
                        }
                        // Error when deleting a message
                        Poll::Ready(Err(err)) => {
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
                                        };
                                    return Poll::Ready(Some(Err(
                                        Error::DeleteStreamMessageAfterCompressErrorBeforeClose(
                                            err,
                                        ),
                                    )));
                                }
                                // The delete list is empty
                                None => {
                                    // Nothing to yield so close stream

                                    // Set next state and return error
                                    *this.state = State::CloseStream;
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
