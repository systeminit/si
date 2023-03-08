//! This library contains tools for creating subscriptions to [NATS](https://nats.io) with native
//! Rust types.

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
}

type SubscriberResult<T> = Result<T, SubscriberError>;

/// Contains the Rust type expected in the subscription stream.
#[derive(Debug)]
pub struct Request<T> {
    /// The Rust type expected in the subscription stream.
    pub payload: T,
}

impl<T> Request<T> {
    pub fn into_parts(self) -> T {
        self.payload
    }
}

pin_project! {
    /// A subscription corresponding to a [NATS](https://nats.io) subject.
    #[derive(Debug)]
    pub struct Subscription<T> {
        #[pin]
        inner: si_data_nats::Subscription,
        _phantom: PhantomData<T>,
    }
}

impl<T> Subscription<T> {
    /// Create a new [`subscription`](Self) for a given request shape `T`. If a queue name is
    /// provided, we will use [`NatsClient::queue_subscribe`] instead of
    /// [`NatsClient::subscribe`].
    pub async fn new(
        nats: &NatsClient,
        subject: impl Into<String>,
        queue_name: Option<impl Into<String>>,
    ) -> SubscriberResult<Subscription<T>> {
        let inner = if let Some(queue_name) = queue_name {
            nats.queue_subscribe(subject, queue_name)
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        } else {
            nats.subscribe(subject)
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        };

        Ok(Subscription {
            inner,
            _phantom: PhantomData::<T>,
        })
    }

    #[allow(dead_code)]
    pub async fn drain(&self) -> SubscriberResult<()> {
        self.inner.drain().await.map_err(SubscriberError::NatsDrain)
    }

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
                let data = nats_msg.into_data();
                let payload: T = match serde_json::from_slice(&data) {
                    // Deserializing from JSON into a formal request type was successful
                    Ok(request) => request,
                    // Deserializing failed
                    Err(err) => {
                        return Poll::Ready(Some(Err(SubscriberError::JSONDeserialize(err))));
                    }
                };

                // Return the request type
                Poll::Ready(Some(Ok(Request { payload })))
            }
            // A NATS error occured (async error or other i/o)
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(SubscriberError::NatsIo(err)))),
            // We see no more messages on the subject, so close the stream
            Poll::Ready(None) => Poll::Ready(None),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}
