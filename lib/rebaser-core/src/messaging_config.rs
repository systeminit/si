//! This module contains [`RebaserMessagingConfig`], which is the config used for the messaging layer between "rebaser"
//! servers and clients.

use serde::{Deserialize, Serialize};

/// A config used for the messaging layer between "rebaser" servers and clients.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RebaserMessagingConfig {
    subject_prefix: Option<String>,
}

impl RebaserMessagingConfig {
    /// Creates a new [`messaging config`](RebaserMessagingConfig).
    pub fn new(subject_prefix: Option<String>) -> Self {
        Self { subject_prefix }
    }

    /// The subject prefix used for creating, using and deleting
    /// [NATS Jetstream](https://docs.nats.io/nats-concepts/jetstream) streams.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.subject_prefix.as_deref()
    }

    /// Sets the subject prefix on the config.
    pub fn set_subject_prefix(&mut self, subject_prefix: impl Into<String>) -> &mut Self {
        self.subject_prefix = Some(subject_prefix.into());
        self
    }
}
