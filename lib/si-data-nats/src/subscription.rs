use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::Stream;
use telemetry::prelude::*;

use super::{ConnectionMetadata, Error, Message, Result};

/// A `Subscription` receives `Message`s published to specific NATS `Subject`s.
#[derive(Debug)]
pub struct Subscription {
    inner: async_nats::Subscriber,
    #[allow(dead_code)]
    shutdown_tx: crossbeam_channel::Sender<()>,
    #[allow(dead_code)]
    shutdown_rx: crossbeam_channel::Receiver<()>,
    metadata: Arc<SubscriptionMessageMetadata>,
    sub_span: Span,
}

impl Subscription {
    pub(crate) fn new(
        inner: async_nats::Subscriber,
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
            shutdown_tx,
            shutdown_rx,
            metadata: Arc::new(metadata),
            sub_span,
        }
    }

    /// Unsubscribe a subscription draining all remaining messages
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
    pub async fn unsubscribe_after(mut self, unsub_after: u64) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        self.inner
            .unsubscribe_after(unsub_after)
            .await
            .map_err(|err| span.record_err(Error::NatsUnsubscribe(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Unsubscribes from subscription after reaching given number of messages. This is the total number of messages received by this subscription in it’s whole lifespan. If it already reached or surpassed the passed value, it will immediately stop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let sub = nc.subscribe("foo").await?;
    /// sub.unsubscribe_after(0).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "subscription.unsubscribe_after",
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
    pub async fn unsubscribe(mut self) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        self.inner
            .unsubscribe()
            .await
            .map_err(|err| span.record_err(Error::NatsUnsubscribe(err)))?;

        span.record_ok();
        Ok(())
    }

    pub fn shutdown(&mut self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Gets a reference to the subscription's span.
    pub fn span(&self) -> &Span {
        &self.sub_span
    }

    /// Gets a reference to the subscription's metadata.
    pub fn metadata(&self) -> &SubscriptionMessageMetadata {
        self.metadata.as_ref()
    }
}

impl Stream for Subscription {
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Some(Message::new(
                msg,
                self.metadata.as_connection_metadata(),
            ))),
            Poll::Ready(None) => Poll::Ready(None),
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
