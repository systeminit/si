use async_nats::{StatusCode, Subject};
use bytes::Bytes;

use super::{ConnectionMetadata, HeaderMap};
use std::{fmt, sync::Arc};

pub use async_nats::Message as InnerMessage;

#[derive(Clone)]
pub struct Message {
    inner: InnerMessage,
    metadata: Arc<ConnectionMetadata>,
}

impl Message {
    pub(crate) fn new(inner: async_nats::Message, metadata: Arc<ConnectionMetadata>) -> Self {
        Self { inner, metadata }
    }

    /// Gets a reference to the [`Subject`] to which this message is published to.
    pub fn subject(&self) -> &Subject {
        &self.inner.subject
    }

    /// Gets a reference to the optional reply [`Subject`] to which responses can be published by
    /// [crate::Subscriber].
    ///
    /// Used for request-response pattern with [crate::Client::request].
    pub fn reply(&self) -> Option<&Subject> {
        self.inner.reply.as_ref()
    }

    /// Gets a reference to the payload of the message.
    ///
    /// Can be any arbitrary format.
    pub fn payload(&self) -> &Bytes {
        &self.inner.payload
    }

    /// Gets a reference to the optional headers.
    pub fn headers(&self) -> Option<&HeaderMap> {
        self.inner.headers.as_ref()
    }

    /// Gets optional [`StatusCode`] of the message.
    ///
    /// Used mostly for internal handling.
    pub fn status(&self) -> Option<StatusCode> {
        self.inner.status
    }

    /// Gets a referrence to optional [`StatusCode`] description.
    pub fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    /// Gets length.
    //
    // NOTE(fnichol): there are no docs upstream, so this appears to be the length of the payload,
    // although maybe it's the entire message?
    pub fn length(&self) -> usize {
        self.inner.length
    }
}

impl Message {
    /// Consumes the message and returns the inner message and connection metadata.
    #[must_use]
    pub fn into_parts(self) -> (InnerMessage, Arc<ConnectionMetadata>) {
        (self.inner, self.metadata)
    }

    /// Get a reference to the connection metadata.
    pub fn metadata(&self) -> Arc<ConnectionMetadata> {
        self.metadata.clone()
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
