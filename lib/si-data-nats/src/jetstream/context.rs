use async_nats::jetstream::stream::{Config, Stream};
use async_nats::{jetstream, HeaderMap};
use bytes::Bytes;

use crate::jetstream::{Consumer, JetstreamError, JetstreamResult};
use crate::Client;
use crate::HeaderName;

/// Used in a [`HeaderMap`] for consumers of stream to reply back to requesters (via a reply mailbox and outside of the
/// stream).
pub static REPLY_SUBJECT_HEADER_NAME: HeaderName = HeaderName::from_static("X-Reply-Subject");

const DEFAULT_MAX_MESSAGES: i64 = 10_000;

/// A wrapper around [`jetstream::Context`].
#[derive(Debug)]
pub struct Context {
    inner: jetstream::Context,
}

impl From<Client> for Context {
    fn from(value: Client) -> Self {
        Self::new(value)
    }
}

impl Context {
    /// Creates a new [`Context`].
    pub fn new(client: Client) -> Self {
        Self {
            inner: jetstream::new(client.inner),
        }
    }

    /// Finds or creates a stream.
    pub async fn get_or_create_stream(
        &self,
        name: impl Into<String>,
        subjects: Vec<String>,
    ) -> JetstreamResult<Stream> {
        // Validate the name before getting or creating the stream. We perform pre-validation to prevent a potential
        // hang.
        let name = name.into();
        validate_stream_subject_name(&name)?;

        let stream = self
            .inner
            .get_or_create_stream(Config {
                name,
                subjects,
                max_messages: DEFAULT_MAX_MESSAGES,
                // TODO(nick): for the rebaser, we must have a work queue retention policy. This is temporary, just for
                // the first pass. I hope this comment doesn't come back to haunt me.
                // retention: RetentionPolicy::WorkQueue,
                ..Default::default()
            })
            .await?;
        Ok(stream)
    }

    /// Publishes to a stream with a reply mailbox in the headers. Immediately awaits the future that the server has
    /// acknowledged the message.
    pub async fn publish_with_reply_mailbox_and_immediately_ack(
        &self,
        client: &Client,
        subject: String,
        bytes: Bytes,
    ) -> JetstreamResult<String> {
        let mut headers = HeaderMap::new();
        let reply_subject = client.new_inbox();
        headers.insert(REPLY_SUBJECT_HEADER_NAME.clone(), reply_subject.as_str());

        let ack_future = self
            .inner
            .publish_with_headers(subject, headers, bytes)
            .await?;

        ack_future.await?;

        Ok(reply_subject)
    }

    /// Finds or creates a [`Consumer`] with a durable, pull configuration. Because the consumer is durable, the server
    /// will remember the last events "acked" in the event of a failure.
    pub async fn get_or_create_durable_consumer(
        &self,
        stream: &Stream,
        name: impl Into<String>,
    ) -> JetstreamResult<Consumer> {
        let name = name.into();
        let raw_consumer = stream
            .get_or_create_consumer(
                name.as_str(),
                jetstream::consumer::pull::Config {
                    durable_name: Some(name.clone()),
                    ..Default::default()
                },
            )
            .await?;
        Ok(Consumer::new(raw_consumer))
    }
}

// This is an opinionated version of the rules from upstream. For example, alphanumeric characters are "recommended",
// but we use "-" for subject prefixes for testing since we cannot use "." (it is prohibited and you will fail stream
// creation).
//
// Link to upstream docs: https://docs.nats.io/running-a-nats-service/nats_admin/jetstream_admin/naming
fn validate_stream_subject_name(subject: impl AsRef<str>) -> JetstreamResult<()> {
    let subject = subject.as_ref();
    for char in subject.chars() {
        let valid = match char {
            ' ' | '.' | '>' | '*' | '/' | '\\' => false,
            char if char != '-' && !char.is_alphanumeric() => false,
            _ => true,
        };
        if !valid {
            return Err(JetstreamError::InvalidSubjectName(subject.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_stream_subject_name() {
        assert!(validate_stream_subject_name("poop.canoe").is_err());
        assert!(validate_stream_subject_name("poop").is_ok());
        assert!(validate_stream_subject_name("poop-canoe").is_ok());
        assert!(validate_stream_subject_name("poop canoe").is_err());
        assert!(validate_stream_subject_name("poop/canoe").is_err());
    }
}
