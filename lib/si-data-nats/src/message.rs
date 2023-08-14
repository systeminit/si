use super::{ConnectionMetadata, HeaderMap};
use std::{fmt, sync::Arc};

#[derive(Clone)]
pub struct Message {
    inner: async_nats::Message,
    metadata: Arc<ConnectionMetadata>,
}

impl Message {
    pub(crate) fn new(inner: async_nats::Message, metadata: Arc<ConnectionMetadata>) -> Self {
        Self { inner, metadata }
    }

    /// Gets a reference to the subject of this message.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.inner.subject
    }

    /// Gets a reference to the reply of this message.
    #[must_use]
    pub fn reply(&self) -> Option<&str> {
        self.inner.reply.as_deref()
    }

    /// Consumes the message and returns the inner data and reply subject.
    #[must_use]
    pub fn into_parts(self) -> (Vec<u8>, Option<String>) {
        (self.inner.payload.into(), self.inner.reply)
    }

    /// Gets a reference to the message contents.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.inner.payload
    }

    /// Gets a reference to the headers of this message.
    #[must_use]
    pub fn headers(&self) -> Option<&HeaderMap> {
        self.inner.headers.as_ref()
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
