use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{FutureExt, Stream};
use telemetry::prelude::*;
use tokio::task::{spawn_blocking, JoinHandle};

use super::{ConnectionMetadata, Error, Message, Result};

/// A `Subscription` receives `Message`s published to specific NATS `Subject`s.
#[derive(Debug)]
pub struct Subscription {
    inner: nats_client::Subscription,
    next: Option<NextMessage>,
    #[allow(dead_code)]
    shutdown_tx: crossbeam_channel::Sender<()>,
    #[allow(dead_code)]
    shutdown_rx: crossbeam_channel::Receiver<()>,
    metadata: Arc<SubscriptionMessageMetadata>,
    sub_span: Span,
}

impl Subscription {
    pub(crate) fn new(
        inner: nats_client::Subscription,
        subject: String,
        connection_metadata: Arc<ConnectionMetadata>,
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

        let metadata = SubscriptionMessageMetadata {
            connection_metadata,
            messaging_destination: subject.clone(),
            messaging_destination_kind: "topic",
            messaging_operation: "process",
            messaging_subject: subject.clone(),
            process_otel_kind: FormattedSpanKind(SpanKind::Consumer).to_string(),
            process_otel_name: format!("{subject} process"),
        };

        Self {
            inner,
            next: None,
            shutdown_tx,
            shutdown_rx,
            metadata: Arc::new(metadata),
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
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let sub = nc.subscribe("foo").await?;
    /// sub.unsubscribe().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "subscription.unsubscribe",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol(),
            messaging.system = %self.metadata.messaging_system(),
            messaging.url = %self.metadata.messaging_url(),
            net.transport = %self.metadata.net_transport(),
            otel.kind = %FormattedSpanKind(SpanKind::Client),
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
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let sub = nc.subscribe("foo").await?;
    /// sub.close().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "subscription.close",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol(),
            messaging.system = %self.metadata.messaging_system(),
            messaging.url = %self.metadata.messaging_url(),
            net.transport = %self.metadata.net_transport(),
            otel.kind = %FormattedSpanKind(SpanKind::Client),
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
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
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
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol(),
            messaging.system = %self.metadata.messaging_system(),
            messaging.url = %self.metadata.messaging_url(),
            net.transport = %self.metadata.net_transport(),
            otel.kind = %FormattedSpanKind(SpanKind::Client),
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

    /// Gets a reference to the subscription's span.
    pub fn span(&self) -> &Span {
        &self.sub_span
    }

    /// Gets a reference to the subscription's metadata.
    pub fn metadata(&self) -> &SubscriptionMessageMetadata {
        self.metadata.as_ref()
    }

    fn next_message(&self) -> NextMessage {
        let inner = self.inner.clone();
        let shutdown_rx = self.shutdown_rx.clone();
        NextMessage {
            handle: spawn_blocking(move || {
                // This blocking code will wait until either the next message arrives, or until a
                // shutdown message is received on the shutdown channel. This select must happen
                // within the blocking code body and therefore must also be a blocking select,
                // hence the reason we're using crossbeam_channel here.
                crossbeam_channel::select! {
                    recv(shutdown_rx) -> _ => {
                        trace!("subscription next message task received shutdown signal");
                        None
                    }
                    recv(inner.receiver()) -> msg_result => {
                        match msg_result {
                            Ok(msg) => Some(msg),
                            Err(err) => {
                                // Unfortunately the underlying blocking API doesn't leave us with
                                // many choices--we're in an error case but all we can do is return
                                // Some(msg) or None and neither is strictly true. The upstream
                                // crate "asynk" impl calls `msg.ok()` which would eat this error
                                // and return None, so instead we're at least going to log this
                                // event in case it causes more trouble. Silent errors are the root
                                // of many long nights on call :(
                                info!(
                                    error = ?err,
                                    concat!(
                                        "crossbeam select error on next message, returning None. ",
                                        "NOTE: this happens normally on subscription shutdown ",
                                        "but should not happen normally."
                                    ),
                                );
                                None
                            }
                        }
                    }
                }
            }),
            metadata: self.metadata.as_connection_metadata(),
        }
    }
}

impl Stream for Subscription {
    type Item = Result<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // If we have a `NextMessage` in progress, grab it and otherwise start a new one
        let mut next = self.next.take().unwrap_or_else(|| self.next_message());

        // Start the subscription span timing--we're doing this after the call above as that was a
        // mutable borrow whereas this is an immutable borrow, and besides the wait time is in the
        // poll, not an in-memory data manipulation
        let entered = self.sub_span.enter();

        // Poll the `NextMessage` future
        match Pin::new(&mut next).poll(cx) {
            // We got a new message, pass it on
            Poll::Ready(Ok(Some(ready))) => Poll::Ready(Some(Ok(ready))),
            // `NextMessage` yielded `None`, meaning the inner subscription is done, so close this
            // stream out
            Poll::Ready(Ok(None)) => Poll::Ready(None),
            // `NextMessage` yielded an error, pass it along
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
            // `NextMessage` returned pending, so stash our future and return pending on the stream
            Poll::Pending => {
                // We're about to mutably borrow to stash our future, so drop the span timing to
                // minimize borrow checker trickery
                drop(entered);
                // Stash the future for the next `poll_next` call
                self.next.replace(next);
                Poll::Pending
            }
        }
    }
}

/// A future that yields the next message in a `Subscription`.
///
/// The implementation wraps an inner blocking API so we're driving a `spawn_blocking` `JoinHandle`
/// to completion.
#[derive(Debug)]
struct NextMessage {
    handle: JoinHandle<Option<nats_client::Message>>,
    metadata: Arc<ConnectionMetadata>,
}

impl Future for NextMessage {
    type Output = Result<Option<Message>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Poll our blocking task handle
        match self.handle.poll_unpin(cx) {
            // Our task completed and yielded a potential message, so convert into our `Message`
            // type and pass it along
            Poll::Ready(Ok(maybe_msg)) => Poll::Ready(Ok(
                maybe_msg.map(|inner| Message::new(inner, self.metadata.clone()))
            )),
            // We got an error--a join handle error from tokio--so return that
            Poll::Ready(Err(err)) => Poll::Ready(Err(Error::Async(err))),
            // Our task is not yet complete, so return pending
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubscriptionMessageMetadata {
    connection_metadata: Arc<ConnectionMetadata>,
    messaging_destination: String,
    messaging_destination_kind: &'static str,
    messaging_operation: &'static str,
    messaging_subject: String,

    process_otel_kind: String,
    process_otel_name: String,
}

impl SubscriptionMessageMetadata {
    /// Gets a reference to the subscription message metadata's messaging consumer id.
    pub fn messaging_consumer_id(&self) -> &str {
        self.connection_metadata.messaging_consumer_id()
    }

    /// Gets a reference to the subscription message metadata's messaging destination.
    pub fn messaging_destination(&self) -> &str {
        self.messaging_destination.as_ref()
    }

    /// Gets a reference to the subscription message metadata's messaging destination kind.
    pub fn messaging_destination_kind(&self) -> &str {
        self.messaging_destination_kind
    }

    /// Gets a reference to the subscription message metadata's messaging operation.
    pub fn messaging_operation(&self) -> &str {
        self.messaging_operation
    }

    /// Gets a reference to the subscription message metadata's messaging protocol.
    pub fn messaging_protocol(&self) -> &str {
        self.connection_metadata.messaging_protocol()
    }

    /// Gets a reference to the subscription message metadata's messaging system.
    pub fn messaging_system(&self) -> &str {
        self.connection_metadata.messaging_system()
    }

    /// Gets a reference to the subscription message metadata's messaging url.
    pub fn messaging_url(&self) -> &str {
        self.connection_metadata.messaging_url()
    }

    /// Gets a reference to the subscription message metadata's messaging subject.
    pub fn messaging_subject(&self) -> &str {
        self.messaging_subject.as_ref()
    }

    /// Get a reference to the subscription message metadata's net transport.
    pub fn net_transport(&self) -> &str {
        self.connection_metadata.net_transport()
    }

    /// Gets a reference to the subscription message metadata's process otel kind.
    pub fn process_otel_kind(&self) -> &str {
        &self.process_otel_kind
    }

    /// Gets a reference to the subscription message metadata's process otel name.
    pub fn process_otel_name(&self) -> &str {
        self.process_otel_name.as_ref()
    }

    /// Get a reference to the subscription message metadata's connection metadata.
    pub fn as_connection_metadata(&self) -> Arc<ConnectionMetadata> {
        self.connection_metadata.clone()
    }
}
