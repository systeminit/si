//! This library contains tools for creating subscribers to [NATS](https://nats.io) with native
//! Rust types.

#![warn(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod builder;

use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use futures_lite::future::FutureExt;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use si_data_nats::NatsError;
use telemetry::prelude::*;
use thiserror::Error;

pub use crate::builder::SubscriberBuilder;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SubscriberError {
    #[error("failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("failed to drain from nats subscriber")]
    NatsDrain(#[source] NatsError),
    #[error("nats io error when reading from subscriber")]
    NatsIo(#[source] NatsError),
    #[error("failed to subscribe to nats topic")]
    NatsSubscribe(#[source] NatsError),
    #[error("failed to unsubscribe from nats subscriber")]
    NatsUnsubscribe(#[source] NatsError),
    #[error("no return mailbox specified; bug! message data: {0:?}")]
    NoReplyMailbox(Vec<u8>),
    #[error("the nats subscriber closed before seeing a final message (expected key: {0})")]
    UnexpectedNatsSubscriberClosed(String),
}

type SubscriberResult<T> = Result<T, SubscriberError>;

/// Contains the Rust type expected in the subscriber stream.
#[derive(Debug)]
pub struct Request<T> {
    /// The Rust type expected in the subscriber stream.
    pub payload: T,
    /// An optional reply mailbox.
    pub reply_mailbox: Option<String>,
}

impl<T> Request<T> {
    /// Split the [`request`](Self)'s fields into individual values.
    pub fn into_parts(self) -> (T, Option<String>) {
        (self.payload, self.reply_mailbox)
    }
}

pin_project! {
    /// A subscriber corresponding to a [NATS](https://nats.io) subject.
    #[derive(Debug)]
    pub struct Subscriber<T> {
        #[pin]
        inner: si_data_nats::Subscriber,
        _phantom: PhantomData<T>,
        subject: String,
        final_message_header_key: Option<String>,
        check_for_reply_mailbox: bool,
    }
}

impl<T> Subscriber<T> {
    /// Provides the [`builder`](SubscriberBuilder) for creating a [`Subscriber`].
    pub fn create(subject: impl Into<String>) -> SubscriberBuilder<T> {
        SubscriberBuilder::new(subject)
    }

    /// Unsubscribe from [NATS](https://nats.io) draining all messages.
    ///
    /// # Errors
    ///
    /// Returns [`SubscriberError`] if the [`Subscriber`] does not successfully unsubscribe.
    pub async fn unsubscribe(self) -> SubscriberResult<()> {
        self.inner
            .unsubscribe()
            .await
            .map_err(SubscriberError::NatsUnsubscribe)
    }

    /// Unsubscribe from [NATS](https://nats.io) after draining some messages
    ///
    /// # Errors
    ///
    /// Returns [`SubscriberError`] if the [`Subscriber`] does not successfully unsubscribe.
    pub async fn unsubscribe_after(self, unsub_after: u64) -> SubscriberResult<()> {
        self.inner
            .unsubscribe_after(unsub_after)
            .await
            .map_err(SubscriberError::NatsUnsubscribe)
    }

    /// Returns the NATS subject to which this subscriber is subscribed.
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl<T> Stream for Subscriber<T>
where
    T: DeserializeOwned,
{
    type Item = SubscriberResult<Request<T>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.inner.next().poll(cx) {
            // Convert this NATS message into the request type `T` and return any errors
            // for the caller to decide how to proceed (i.e. does the caller fail on first error,
            // ignore error items, etc.)
            Poll::Ready(Some(nats_msg)) => {
                // Only check if the message has a final message header if our subscriber config
                // specified one (or used the default).
                if let Some(final_message_header_key) = this.final_message_header_key {
                    // If the NATS message has a final message header, then treat this as an
                    // end-of-stream marker and close our stream.
                    if let Some(headers) = nats_msg.headers() {
                        if headers
                            .iter()
                            .any(|(key, _)| AsRef::<str>::as_ref(key) == final_message_header_key)
                        {
                            trace!(
                                "{} header detected in NATS message, closing stream",
                                final_message_header_key
                            );
                            return Poll::Ready(None);
                        }
                    }
                }

                let (data, reply) = nats_msg.into_parts();
                let reply_mailbox = reply;

                // Always provide the reply_mailbox if there is one, but only make it an error if
                // we were told to explicitly check for one.
                if *this.check_for_reply_mailbox && reply_mailbox.is_none() {
                    return Poll::Ready(Some(Err(SubscriberError::NoReplyMailbox(data))));
                }

                let payload: T = match serde_json::from_slice(&data) {
                    // Deserializing from JSON into a formal request type was successful
                    Ok(request) => request,
                    // Deserializing failed
                    Err(err) => {
                        return Poll::Ready(Some(Err(SubscriberError::JSONDeserialize(err))));
                    }
                };

                // Return the request type
                Poll::Ready(Some(Ok(Request {
                    payload,
                    reply_mailbox,
                })))
            }
            // We see no more messages on the subject, so let's decide what to do
            Poll::Ready(None) => match this.final_message_header_key {
                // If we are expecting a "final message" header key, then this is an unexpected
                // problem
                Some(key) => Poll::Ready(Some(Err(
                    SubscriberError::UnexpectedNatsSubscriberClosed(key.to_string()),
                ))),
                // If we are not expecting a "final message" header key, then we can successfully
                // close the stream
                None => Poll::Ready(None),
            },
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}
