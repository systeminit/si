use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use deadpool_cyclone::{QualificationCheckRequest, ResolverFunctionRequest, ResourceSyncRequest};
use futures::{Stream, StreamExt};
use futures_lite::FutureExt;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use si_data::{nats, NatsClient};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubscriberError {
    #[error("failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("failed to drain from nats subscription")]
    NatsDrain(#[source] si_data::NatsError),
    #[error("nats io error when reading from subscription")]
    NatsIo(#[source] si_data::NatsError),
    #[error("failed to subscribe to nats topic")]
    NatsSubscribe(#[source] si_data::NatsError),
    #[error("failed to unsubscribe from nats subscription")]
    NatsUnsubscribe(#[source] si_data::NatsError),
    #[error("no return mailbox specified; bug! message data: {0:?}")]
    NoReplyMailbox(Vec<u8>),
}

type Result<T> = std::result::Result<T, SubscriberError>;

pub struct Subscriber;

impl Subscriber {
    pub async fn qualification_check(
        nats: &NatsClient,
    ) -> Result<Subscription<QualificationCheckRequest>> {
        let inner = nats
            .subscribe("veritech.function.qualification")
            .await
            .map_err(SubscriberError::NatsSubscribe)?;

        Ok(Subscription {
            inner,
            _phantom: PhantomData,
        })
    }

    pub async fn resolver_function(
        nats: &NatsClient,
    ) -> Result<Subscription<ResolverFunctionRequest>> {
        let inner = nats
            .subscribe("veritech.function.resolver")
            .await
            .map_err(SubscriberError::NatsSubscribe)?;

        Ok(Subscription {
            inner,
            _phantom: PhantomData,
        })
    }

    pub async fn resource_sync(nats: &NatsClient) -> Result<Subscription<ResourceSyncRequest>> {
        let inner = nats
            .subscribe("veritech.function.sync")
            .await
            .map_err(SubscriberError::NatsSubscribe)?;

        Ok(Subscription {
            inner,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Request<T> {
    reply_mailbox: String,
    cyclone_request: T,
}

impl<T> Request<T> {
    pub fn into_parts(self) -> (String, T) {
        (self.reply_mailbox, self.cyclone_request)
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct Subscription<T> {
        #[pin]
        inner: nats::Subscription,
        _phantom: PhantomData<T>,
    }
}

impl<T> Subscription<T> {
    #[allow(dead_code)]
    pub async fn drain(&self) -> Result<()> {
        self.inner.drain().await.map_err(SubscriberError::NatsDrain)
    }

    pub async fn unsubscribe(self) -> Result<()> {
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
    type Item = Result<Request<T>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.inner.next().poll(cx) {
            // Convert this NATS message into the cyclone request type `T` and return any errors
            // for the caller to decide how to proceed (i.e. does the caller fail on first error,
            // ignore error items, etc.)
            Poll::Ready(Some(Ok(nats_msg))) => {
                let (data, reply) = nats_msg.into_parts();
                let reply_mailbox = match reply {
                    // We have a reply mailbox, good
                    Some(reply) => reply,
                    // No reply mailbox provided
                    None => return Poll::Ready(Some(Err(SubscriberError::NoReplyMailbox(data)))),
                };
                let cyclone_request: T = match serde_json::from_slice(&data) {
                    // Deserializing from JSON into a formal request type was successful
                    Ok(request) => request,
                    // Deserializing failed
                    Err(err) => {
                        return Poll::Ready(Some(Err(SubscriberError::JSONDeserialize(err))))
                    }
                };
                let request = Request {
                    reply_mailbox,
                    cyclone_request,
                };

                // Return the request type
                Poll::Ready(Some(Ok(request)))
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
