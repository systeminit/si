//! This library contains tools for creating subscriptions to [NATS](https://nats.io) with native
//! Rust types.

#![warn(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use futures_lite::future::FutureExt;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use si_data_nats::{NatsClient, NatsError};
use telemetry::prelude::*;
use thiserror::Error;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum SubscriberError {
    #[error("failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("failed to drain from nats subscription")]
    NatsDrain(#[source] NatsError),
    #[error("nats io error when reading from subscription")]
    NatsIo(#[source] NatsError),
    #[error("failed to subscribe to nats topic")]
    NatsSubscribe(#[source] NatsError),
    #[error("failed to unsubscribe from nats subscription")]
    NatsUnsubscribe(#[source] NatsError),
    #[error("the nats subscription closed before seeing a final message (expected key: {0})")]
    UnexpectedNatsSubscriptionClosed(String),
    #[error("no return mailbox specified; bug! message data: {0:?}")]
    NoReplyMailbox(Vec<u8>),
}

type SubscriberResult<T> = Result<T, SubscriberError>;

/// The default "final message" header key that corresponds to
/// [`SubscriptionConfigKeyOption::UseDefaultKey`].
pub const DEFAULT_FINAL_MESSAGE_HEADER_KEY: &str = "X-Final-Message";

/// Contains the Rust type expected in the subscription stream.
#[derive(Debug)]
pub struct Request<T> {
    /// The Rust type expected in the subscription stream.
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
    /// A subscription corresponding to a [NATS](https://nats.io) subject.
    #[derive(Debug)]
    pub struct Subscription<T> {
        #[pin]
        inner: si_data_nats::Subscription,
        _phantom: PhantomData<T>,
        final_message_header_key: Option<String>,
        check_for_reply_mailbox: bool,
    }
}

/// The [`config`](Self) used for creating a [`Subscription`].
pub struct SubscriptionConfig<S>
where
    S: Into<String>,
{
    /// The [NATS](https://nats.io) subject used.
    pub subject: S,
    /// If provided, the [`Subscription`] will use [`NatsClient::queue_subscribe`]. Otherwise, it
    /// [`NatsClient::subscribe`].
    pub queue_name: Option<S>,
    /// If a key is provided (or the [`default`](DEFAULT_FINAL_MESSAGE_HEADER_KEY) is used), the
    /// [`Subscription`] will only close successfully if a "final message" is seen. Otherwise, it
    /// can close successfully without receiving a "final message".
    pub final_message_header_key: SubscriptionConfigKeyOption<S>,
    /// If set, the [`Subscription`] will check for a reply mailbox in the [`Request`].
    /// Otherwise, it will not perform the check.
    pub check_for_reply_mailbox: bool,
}

/// Clarifies whether or not to use a "final message" header key for successfully closing
/// [`Subscriptions`](Subscription) (as well as what key should be used, if applicable).
pub enum SubscriptionConfigKeyOption<S>
where
    S: Into<String>,
{
    /// Use the provided key.
    UseKey(S),
    /// Use the default key: [`DEFAULT_FINAL_MESSAGE_HEADER_KEY`].
    UseDefaultKey,
    /// Do not use a key (i.e. allow successful close without seeing a "final message").
    DoNotUseKey,
}

impl<T> Subscription<T> {
    /// Create a new [`subscription`](Self) for a given request shape `T`.
    ///
    /// # Errors
    ///
    /// Returns [`SubscriberError`] if a [`Subscription`] could not be created.
    pub async fn new(
        nats: &NatsClient,
        config: SubscriptionConfig<impl Into<String>>,
    ) -> SubscriberResult<Subscription<T>> {
        let inner = if let Some(queue_name) = config.queue_name {
            nats.queue_subscribe(config.subject, queue_name)
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        } else {
            nats.subscribe(config.subject)
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        };

        let final_message_header_key = match config.final_message_header_key {
            SubscriptionConfigKeyOption::UseKey(key) => Some(key.into()),
            SubscriptionConfigKeyOption::UseDefaultKey => {
                Some(DEFAULT_FINAL_MESSAGE_HEADER_KEY.into())
            }
            SubscriptionConfigKeyOption::DoNotUseKey => None,
        };

        Ok(Subscription {
            inner,
            _phantom: PhantomData::<T>,
            final_message_header_key,
            check_for_reply_mailbox: config.check_for_reply_mailbox,
        })
    }

    #[allow(dead_code, missing_docs, clippy::missing_errors_doc)]
    pub async fn drain(&self) -> SubscriberResult<()> {
        self.inner.drain().await.map_err(SubscriberError::NatsDrain)
    }

    /// Unsubscribe from [NATS](https://nats.io).
    ///
    /// # Errors
    ///
    /// Returns [`SubscriberError`] if the [`Subscription`] does not successfully unsubscribe.
    pub async fn unsubscribe(self) -> SubscriberResult<()> {
        self.inner
            .unsubscribe()
            .await
            .map_err(SubscriberError::NatsUnsubscribe)
    }
}

impl<T> Stream for Subscription<T>
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
            Poll::Ready(Some(Ok(nats_msg))) => {
                // Only check if the message has a final message header if our subscription config
                // specified one (or used the default).
                if let Some(final_message_header_key) = this.final_message_header_key {
                    // If the NATS message has a final message header, then treat this as an
                    // end-of-stream marker and close our stream.
                    if let Some(headers) = nats_msg.headers() {
                        if headers.keys().any(|key| key == final_message_header_key) {
                            trace!(
                                "{} header detected in NATS message, closing stream",
                                final_message_header_key
                            );
                            return Poll::Ready(None);
                        }
                    }
                }

                let (data, reply) = nats_msg.into_parts();

                let reply_mailbox = match this.check_for_reply_mailbox {
                    true => match reply {
                        // We have a reply mailbox, good
                        Some(reply) => Some(reply),
                        // No reply mailbox provided
                        None => {
                            return Poll::Ready(Some(Err(SubscriberError::NoReplyMailbox(data))));
                        }
                    },
                    // If we do not have to check the reply mailbox, use "None"
                    false => None,
                };

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
            // A NATS error occurred (async error or other i/o)
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(SubscriberError::NatsIo(err)))),
            // We see no more messages on the subject, so let's decide what to do
            Poll::Ready(None) => match this.final_message_header_key {
                // If we are expecting a "final message" header key, then this is an unexpected
                // problem
                Some(key) => Poll::Ready(Some(Err(
                    SubscriberError::UnexpectedNatsSubscriptionClosed(key.to_string()),
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
