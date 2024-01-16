//! This module contains [`SubscriberBuilder`], which is used for creating
//! [`Subscribers`](crate::Subscriber).

use std::marker::PhantomData;

use si_data_nats::{subject::ToSubject, NatsClient, Subject};
use telemetry::prelude::*;
use telemetry_nats::NatsMakeSpan;

use crate::{Subscriber, SubscriberError, SubscriberResult};

/// The [`builder`](Self) used for creating a [`Subscriber`].
pub struct SubscriberBuilder<T> {
    /// The [NATS](https://nats.io) subject used.
    pub subject: Subject,
    /// Indicates the final type of the [`Request`](crate::Request).
    _phantom: PhantomData<T>,

    /// If provided, the [`Subscriber`] will use [`NatsClient::queue_subscribe`]. Otherwise, it
    /// [`NatsClient::subscribe`].
    pub queue_name: Option<String>,
    /// If a key is provided, the [`Subscriber`] will only close successfully if a "final message"
    /// is seen. Otherwise, it can close successfully without receiving a "final message".
    pub final_message_header_key: Option<String>,
    /// If set, the [`Subscriber`] will check for a reply mailbox in the
    /// [`Request`](crate::Request).
    /// Otherwise, it will not perform the check.
    pub check_for_reply_mailbox: bool,

    /// The logging level of the message processing spans
    span_level: Level,
}

impl<T> SubscriberBuilder<T> {
    /// Create a new [`builder`](SubscriberBuilder) for building a [`Subscriber`].
    pub fn new(subject: impl ToSubject) -> Self {
        Self {
            subject: subject.to_subject(),
            _phantom: PhantomData::<T>,
            queue_name: None,
            final_message_header_key: None,
            check_for_reply_mailbox: false,
            span_level: Level::INFO,
        }
    }

    /// Start a new [`Subscriber`] for a given [`request`](crate::Request) shape `T`. This will
    /// consume [`Self`].
    ///
    /// # Errors
    ///
    /// Returns [`SubscriberError`] if a [`Subscriber`] could not be created.
    pub async fn start(self, nats: &NatsClient) -> SubscriberResult<Subscriber<T>> {
        let inner = if let Some(queue_name) = self.queue_name {
            nats.queue_subscribe(self.subject.clone(), queue_name)
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        } else {
            nats.subscribe(self.subject.clone())
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        };

        let make_span = NatsMakeSpan::new().level(self.span_level);

        Ok(Subscriber {
            inner,
            _phantom: PhantomData::<T>,
            subject: self.subject,
            final_message_header_key: self.final_message_header_key,
            check_for_reply_mailbox: self.check_for_reply_mailbox,
            make_span,
        })
    }

    /// Sets the "queue_name" field.
    pub fn queue_name(mut self, queue_name: impl Into<String>) -> Self {
        self.queue_name = Some(queue_name.into());
        self
    }

    /// Sets the "final_message_header_key" field.
    pub fn final_message_header_key(mut self, final_message_header_key: impl Into<String>) -> Self {
        self.final_message_header_key = Some(final_message_header_key.into());
        self
    }

    /// Sets the "check_for_reply_mailbox" field.
    pub fn check_for_reply_mailbox(mut self) -> Self {
        self.check_for_reply_mailbox = true;
        self
    }

    /// Sets the logging level for the message processing spans.
    pub fn span_level(mut self, level: Level) -> Self {
        self.span_level = level;
        self
    }
}
