use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{FutureExt, Stream};
use nats_client as nats;
use telemetry::prelude::*;
use tokio::task::spawn_blocking;

use super::{ConnectionMetadata, Error, Message, Result};

/// A `Subscription` receives `Message`s published to specific NATS `Subject`s.
#[derive(Debug)]
pub struct Subscription {
    inner: nats::Subscription,
    shutdown_tx: crossbeam_channel::Sender<()>,
    shutdown_rx: crossbeam_channel::Receiver<()>,
    metadata: Arc<ConnectionMetadata>,
    sub_span: Span,
}

impl Subscription {
    pub(crate) fn new(
        inner: nats::Subscription,
        metadata: Arc<ConnectionMetadata>,
        sub_span: Span,
    ) -> Self {
        // We don't use the tx side explicitly, but rather rely on the behavior when this
        // Subscription is dropped, then tx is closed and the rx side running in a thread will get
        // its shutdown signal immediately afterwards. That way we don't keep a zombie task
        // receiving from a subscription that is no longer valid.
        //
        // This strategy uses a crossbeam channel as it presents a blocking API which we need in
        // our blocking thread work. Bounded with size zero roughly maps to a "oneshot" channel,
        // but with cloneable tx and rx ends.
        let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded(0);

        Self {
            inner,
            shutdown_tx,
            shutdown_rx,
            metadata,
            sub_span,
        }
    }

    /// Unsubscribe a subscription immediately without draining.
    ///
    /// Use `drain` instead if you want any pending messages to be processed by a handler, if one
    /// is configured.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// let sub = nc.subscribe("foo").await?;
    /// sub.unsubscribe().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "subscription.unsubscribe",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn unsubscribe(self) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        spawn_blocking(move || self.inner.unsubscribe())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Close a subscription. Same as `unsubscribe`
    ///
    /// Use `drain` instead if you want any pending messages to be processed by a handler, if one
    /// is configured.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// let sub = nc.subscribe("foo").await?;
    /// sub.close().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "subscription.close",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn close(self) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        spawn_blocking(move || self.inner.close())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Send an unsubscription then flush the connection, allowing any unprocessed messages to be
    /// handled by a handler function if one is configured.
    ///
    /// After the flush returns, we know that a round-trip to the server has happened after it
    /// received our unsubscription, so we shut down the subscriber afterwards.
    ///
    /// A similar method exists on the `Connection` struct which will drain all subscriptions for
    /// the NATS client, and transition the entire system into the closed state afterward.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::StreamExt;
    /// # use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
    /// # use std::thread;
    /// # use std::time::Duration;
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// let mut sub = nc.subscribe("test.drain").await?;
    ///
    /// nc.publish("test.drain", "message").await?;
    /// sub.drain().await?;
    ///
    /// let mut received = false;
    /// while sub.next().await.is_some() {
    ///     received = true;
    /// }
    ///
    /// assert!(received);
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "subscription.drain",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn drain(&self) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        let inner = self.inner.clone();
        spawn_blocking(move || inner.drain())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }
}

impl Stream for Subscription {
    type Item = Result<Message>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let _enter = self.sub_span.enter();
        let inner = self.inner.clone();
        match spawn_blocking(move || inner.next()).poll_unpin(cx) {
            Poll::Ready(Ok(ready)) => Poll::Ready(
                Ok(ready.map(|msg| Message::new(msg, self.metadata.clone()))).transpose(),
            ),
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(Error::Async(err)))),
            Poll::Pending => Poll::Pending,
        }
    }
}
