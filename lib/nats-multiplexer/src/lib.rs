//! This crate contains [`Multiplexer`], which provides the ability to only use one [NATS](https://nats.io)
//! subscription (usually with wildcard(s) in the subject) to manage receiving on multiple channels.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::{
    collections::HashMap,
    fmt::Debug,
};

use futures::StreamExt;
use nats_multiplexer_client::MultiplexerClient;
use nats_multiplexer_core::{
    MultiplexerKey,
    MultiplexerRequest,
    MultiplexerRequestPayload,
};
use si_data_nats::{
    Message,
    NatsClient,
    OpenTelemetryContext,
    Subject,
    Subscriber,
    subject::ToSubject,
};
use telemetry::{
    OpenTelemetrySpanExt,
    prelude::*,
};
use telemetry_nats::propagation;
use thiserror::Error;
use tokio::sync::{
    broadcast,
    mpsc,
};
use tokio_util::sync::CancellationToken;

/// The buffer used for senders within the [`Multiplexer's`] channels map.
const MULTIPLEXER_BROADCAST_SENDER_BUFFER: usize = 4096;

// NOTE(nick): this module is intentionally private.
mod parsing;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum MultiplexerError {
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
}

#[allow(missing_docs)]
pub type MultiplexerResult<T> = Result<T, MultiplexerError>;

/// A [NATS](https://nats.io) multiplexer, will contains a subscription to one subject and contains a map of channels
/// for those wishing to receive from the same or more-specific subjects.
#[derive(Debug)]
pub struct Multiplexer {
    subject: Subject,
    subscriber: Subscriber,
    channels: HashMap<MultiplexerKey, broadcast::Sender<MultiplexerRequestPayload>>,
    client_rx: mpsc::UnboundedReceiver<MultiplexerRequest>,
    token: CancellationToken,
    activate_instrumentation_with_name: Option<String>,
}

impl Multiplexer {
    const NAME: &'static str = "nats_multiplexter::multiplexer";

    /// Creates a new [`Multiplexer`].
    ///
    /// **Warning:** if you add instrumentation, you may produce _many_ spans. Buyer beware!
    pub async fn new(
        nats: &NatsClient,
        subject: impl ToSubject,
        token: CancellationToken,
        activate_instrumentation_with_name: Option<String>,
    ) -> MultiplexerResult<(Self, MultiplexerClient)> {
        let subject = subject.to_subject();
        let subscriber = nats.subscribe(subject.clone()).await?;
        let (client_tx, client_rx) = mpsc::unbounded_channel();
        Ok((
            Self {
                subscriber,
                channels: Default::default(),
                client_rx,
                subject,
                token,
                activate_instrumentation_with_name,
            },
            MultiplexerClient::new(client_tx),
        ))
    }

    /// Runs the [`Multiplexer`] with a given shutdown receiver.
    pub async fn run(mut self) {
        debug!(%self.subject, "running channel multiplexer");

        loop {
            tokio::select! {
                Some(message) = self.subscriber.next() => {
                    match &self.activate_instrumentation_with_name {
                        Some(name) => {
                            if let Err(err) = self.process_message_with_instrumentation(message, name) {
                                error!(
                                    si.error.message = ?err,
                                    "error when processing message in nats multiplexer"
                                );
                            }
                        }
                        None => {
                            if let Err(err) = self.process_message(message, None) {
                                error!(
                                    si.error.message = ?err,
                                    "error when processing message in nats multiplexer"
                                );
                            }
                        }
                    }
                }
                Some(request) = self.client_rx.recv() => {
                    if let Err(err) = self.process_client_request(request) {
                        error!(
                            si.error.message = ?err,
                            "error when processing client request in nats multiplexer"
                        );
                    }
                }
                _ = self.token.cancelled() => {
                    info!(
                        task = Self::NAME,
                        subject = %self.subject,
                        "received cancellation",
                    );

                    // NOTE(nick,fletcher): we may not want to unsubscribe here.
                    if let Err(err) = self.subscriber.unsubscribe().await {
                        error!(
                            si.error.message = ?err,
                            "error when unsubscribing in nats multiplexer"
                        );
                    }
                    break;
                },
            }
        }

        debug!(task = Self::NAME, subject = %self.subject, "shutdown complete");
    }

    #[instrument(name = "nats_multiplexer.multiplexer.process_message", level = "info", skip_all, fields(si.nats_multiplexer.name = %name))]
    fn process_message_with_instrumentation(
        &self,
        message: Message,
        name: &str,
    ) -> MultiplexerResult<()> {
        let span = current_span_for_instrument_at!("info");

        if let Some(headers) = message.headers() {
            span.set_parent(propagation::extract_opentelemetry_context(headers));
        }

        self.process_message(message, Some(span.context()))
    }

    fn process_message(
        &self,
        message: Message,
        otel_ctx: Option<OpenTelemetryContext>,
    ) -> MultiplexerResult<()> {
        let subject = message.subject().to_string();

        // We need to fan out not only to those receiving for the literal subject, but also for those using wildcards.
        // That is just wild!
        for key in parsing::keys_for_potential_receivers(subject.clone()) {
            if let Some(sender) = self.channels.get(&key) {
                trace!(%subject, %key, "sending message for receiver corresponding to key");
                if sender
                    .send(MultiplexerRequestPayload {
                        nats_message: message.clone(),
                        otel_ctx: otel_ctx.clone(),
                    })
                    .is_err()
                {
                    trace!(%subject, %key, "unable to send message (likely there are no receivers left)");
                }
            }
        }
        Ok(())
    }

    fn process_client_request(&mut self, request: MultiplexerRequest) -> MultiplexerResult<()> {
        match request {
            MultiplexerRequest::Add((subject, reply_tx)) => {
                // NOTE(nick): major props to fnichol for this idea.
                let sender = self.channels.entry(subject).or_insert_with(|| {
                    let (sender, _) = broadcast::channel(MULTIPLEXER_BROADCAST_SENDER_BUFFER);
                    sender
                });

                // NOTE(nick): this returns what it couldn't send when erroring.
                if reply_tx.send(sender.subscribe()).is_err() {
                    error!("could not process client request in nats multiplexer");
                }
            }
        }
        Ok(())
    }
}
