use async_nats::jetstream;
use async_nats::jetstream::consumer::pull::{Config, Stream};

use crate::jetstream::JetstreamResult;

/// A wrapper around [`jetstream::consumer::Consumer<Config>`].
#[derive(Debug)]
pub struct Consumer {
    inner: jetstream::consumer::Consumer<Config>,
}

impl Consumer {
    /// Creates a new [`Consumer`].
    pub fn new(raw_consumer: jetstream::consumer::Consumer<Config>) -> Self {
        Self {
            inner: raw_consumer,
        }
    }

    /// Creates a [`Stream`] from self.
    pub async fn stream(&self) -> JetstreamResult<Stream> {
        Ok(self.inner.messages().await?)
    }
}

impl From<jetstream::consumer::Consumer<Config>> for Consumer {
    fn from(value: jetstream::consumer::Consumer<Config>) -> Self {
        Self::new(value)
    }
}
