//! This module contains [`SubscriptionBuilder`], which is used for creating
//! [`Subscriptions`](crate::Subscription).

use std::marker::PhantomData;

use si_data_nats::NatsClient;

use crate::{SubscriberError, SubscriberResult, Subscription};

/// The [`builder`](Self) used for creating a [`Subscription`].
pub struct SubscriptionBuilder<T> {
    /// The [NATS](https://nats.io) subject used.
    pub subject: String,
    /// Indicates the final type of the [`Request`](crate::Request).
    _phantom: PhantomData<T>,

    /// If provided, the [`Subscription`] will use [`NatsClient::queue_subscribe`]. Otherwise, it
    /// [`NatsClient::subscribe`].
    pub queue_name: Option<String>,
    /// If a key is provided, the [`Subscription`] will only close successfully if a "final message"
    /// is seen. Otherwise, it can close successfully without receiving a "final message".
    pub final_message_header_key: Option<String>,
    /// If set, the [`Subscription`] will check for a reply mailbox in the
    /// [`Request`](crate::Request).
    /// Otherwise, it will not perform the check.
    pub check_for_reply_mailbox: bool,
}

impl<T> SubscriptionBuilder<T> {
    /// Create a new [`builder`](SubscriptionBuilder) for building a [`Subscription`].
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            _phantom: PhantomData::<T>,
            queue_name: None,
            final_message_header_key: None,
            check_for_reply_mailbox: false,
        }
    }

    /// Start a new [`Subscription`] for a given [`request`](crate::Request) shape `T`. This will
    /// consume [`Self`].
    ///
    /// # Errors
    ///
    /// Returns [`SubscriberError`] if a [`Subscription`] could not be created.
    pub async fn start(self, nats: &NatsClient) -> SubscriberResult<Subscription<T>> {
        let inner = if let Some(queue_name) = self.queue_name {
            nats.queue_subscribe(self.subject.clone(), queue_name)
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        } else {
            nats.subscribe(self.subject.clone())
                .await
                .map_err(SubscriberError::NatsSubscribe)?
        };

        Ok(Subscription {
            inner,
            _phantom: PhantomData::<T>,
            subject: self.subject,
            final_message_header_key: self.final_message_header_key,
            check_for_reply_mailbox: self.check_for_reply_mailbox,
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
}
