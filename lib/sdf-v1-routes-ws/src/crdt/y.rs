use std::{
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

/// Adapted from: https://github.com/y-crdt/yrs-warp/blob/14a1abdf9085d71b6071e27c3e53ac5d0e07735d/src/ws.rs
use futures::{
    Future,
    Sink,
    Stream,
};
use futures_lite::future::FutureExt;
use nats_multiplexer_client::MultiplexerRequestPayload;
use si_data_nats::{
    NatsClient,
    Subject,
};
use telemetry::prelude::error;
use tokio::sync::broadcast;
use tokio_stream::wrappers::{
    BroadcastStream,
    errors::BroadcastStreamRecvError,
};
use y_sync::sync::Error;

type Result<T, E = Error> = std::result::Result<T, E>;
type BoxedResultFuture<T> = Box<dyn Future<Output = Result<T>> + Sync + Send>;

pub struct YSink {
    nats: NatsClient,
    channel: Subject,
    future: Option<Pin<BoxedResultFuture<()>>>,
}

impl YSink {
    pub fn new(nats: NatsClient, channel: Subject) -> Self {
        Self {
            nats,
            channel,
            future: None,
        }
    }
}

impl Sink<Vec<u8>> for YSink {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, payload: Vec<u8>) -> Result<(), Self::Error> {
        let (nats, channel) = (self.nats.clone(), self.channel.clone());
        self.future = Some(Box::pin(async move {
            nats.publish(channel, payload.into())
                .await
                .map_err(|err| Error::Other(err.into()))
        }));
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.future.take() {
            Some(mut future) => match future.poll(cx) {
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
                Poll::Ready(Err(err)) => Poll::Ready(Err(Error::Other(err.into()))),
                Poll::Pending => {
                    self.future = Some(future);
                    Poll::Pending
                }
            },
            _ => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}

pub struct YStream(BroadcastStream<MultiplexerRequestPayload>);

impl YStream {
    pub fn new(receiver: broadcast::Receiver<MultiplexerRequestPayload>) -> Self {
        Self(BroadcastStream::new(receiver))
    }
}

impl Stream for YStream {
    type Item = Result<Vec<u8>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(payload_result)) => match payload_result {
                Ok(payload) => {
                    Poll::Ready(Some(Ok(payload.nats_message.into_parts().0.payload.into())))
                }
                Err(error) => match error {
                    error @ BroadcastStreamRecvError::Lagged(number_of_missed_messages) => {
                        error!(
                            "found broadcast stream recv error: lagged and missed {number_of_missed_messages} messages"
                        );
                        Poll::Ready(Some(Err(Error::Other(error.into()))))
                    }
                },
            },
        }
    }
}
