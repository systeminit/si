use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use futures_lite::FutureExt;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_core::FINAL_MESSAGE_HEADER_KEY;

#[derive(Error, Debug)]
pub enum SubscriptionError {
    #[error("failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("nats io error when reading from subscription")]
    NatsIo(#[source] si_data_nats::NatsError),
    #[error("failed to unsubscribe from nats subscription")]
    NatsUnsubscribe(#[source] si_data_nats::NatsError),
    #[error("the nats subscription closed before seeing a final message")]
    UnexpectedNatsSubscriptionClosed,
}

pin_project! {
    #[derive(Debug)]
    pub struct Subscription<T> {
        #[pin]
        inner: si_data_nats::Subscription,
        _phantom: PhantomData<T>,
    }
}

impl<T> Subscription<T> {
    pub fn new(inner: si_data_nats::Subscription) -> Self {
        Subscription {
            inner,
            _phantom: PhantomData,
        }
    }

    pub async fn unsubscribe(self) -> Result<(), SubscriptionError> {
        self.inner
            .unsubscribe()
            .await
            .map_err(SubscriptionError::NatsUnsubscribe)
    }
}

impl<T> Stream for Subscription<T>
where
    T: DeserializeOwned,
{
    type Item = Result<T, SubscriptionError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.inner.next().poll(cx) {
            // Convert this NATS message into the cyclone request type `T` and return any
            // errors for the caller to decide how to proceed (i.e. does the caller fail on
            // first error, ignore error items, etc.)
            Poll::Ready(Some(Ok(nats_msg))) => {
                // If the NATS message has a final message header, then treat this as an
                // end-of-stream marker and close our stream.
                if let Some(headers) = nats_msg.headers() {
                    if headers.keys().any(|key| key == FINAL_MESSAGE_HEADER_KEY) {
                        trace!(
                            "{} header detected in NATS message, closing stream",
                            FINAL_MESSAGE_HEADER_KEY
                        );
                        return Poll::Ready(None);
                    }
                }

                let data = nats_msg.into_data();
                match serde_json::from_slice::<T>(&data) {
                    // Deserializing from JSON into the target type was successful
                    Ok(msg) => Poll::Ready(Some(Ok(msg))),
                    // Deserializing failed
                    Err(err) => Poll::Ready(Some(Err(SubscriptionError::JSONDeserialize(err)))),
                }
            }
            // A NATS error occurred (async error or other i/o)
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(SubscriptionError::NatsIo(err)))),
            // We see no more messages on the subject, but we haven't seen a "final message"
            // yet, so this is an unexpected problem
            Poll::Ready(None) => Poll::Ready(Some(Err(
                SubscriptionError::UnexpectedNatsSubscriptionClosed,
            ))),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}
