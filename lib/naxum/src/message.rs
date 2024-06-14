use std::{error, fmt, ops};

use async_nats::{HeaderMap, StatusCode, Subject};
use bytes::Bytes;

mod extensions;

pub use extensions::Extensions;

use crate::response::{IntoResponse, Response};

#[derive(Clone)]
pub struct Message<T> {
    inner: T,
    extensions: Extensions,
}

impl<T> Message<T>
where
    T: MessageHead,
{
    #[inline]
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner,
            extensions: Extensions::new(),
        }
    }

    #[inline]
    pub(crate) fn new_with_extensions(inner: T, extensions: Extensions) -> Self {
        Self { inner, extensions }
    }

    #[inline]
    pub fn from_parts(head: Head, payload: Bytes) -> Result<Self, FromPartsError> {
        let (inner, extensions) = T::from_head_and_payload(head, payload)?;

        Ok(Self::new_with_extensions(inner, extensions))
    }

    #[inline]
    pub fn subject(&self) -> &Subject {
        self.inner.subject()
    }

    #[inline]
    pub fn reply(&self) -> Option<&Subject> {
        self.inner.reply()
    }

    #[inline]
    pub fn headers(&self) -> Option<&HeaderMap> {
        self.inner.headers()
    }

    #[inline]
    pub fn status(&self) -> Option<StatusCode> {
        self.inner.status()
    }

    #[inline]
    pub fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    #[inline]
    pub fn length(&self) -> usize {
        self.inner.length()
    }

    #[inline]
    pub fn payload_length(&self) -> usize {
        self.inner.payload_length()
    }
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub fn head(&self) -> HeadRef<'_> {
        HeadRef {
            subject: self.subject(),
            reply: self.reply(),
            headers: self.headers(),
            status: self.status(),
            description: self.description(),
            length: self.length(),
            payload_length: self.payload_length(),
            extensions: self.extensions(),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }

    #[inline]
    pub fn split(self) -> (T, Extensions) {
        (self.inner, self.extensions)
    }

    #[inline]
    pub fn into_parts(self) -> (Head, Bytes) {
        let (mut head, payload) = self.inner.into_head_and_payload();
        head.extensions.extend(self.extensions);

        (head, payload)
    }
}

impl<T: fmt::Debug> fmt::Debug for Message<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message")
            .field("parts", &self.extensions)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> ops::Deref for Message<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ops::DerefMut for Message<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> From<T> for Message<T>
where
    T: MessageHead,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FromPartsError(&'static str);

impl fmt::Display for FromPartsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to build message from parts: {}", self.0)
    }
}

impl error::Error for FromPartsError {}

impl IntoResponse for FromPartsError {
    fn into_response(self) -> Response {
        Response::default_internal_server_error()
    }
}

pub trait MessageHead {
    /// Subject to which message is published to.
    fn subject(&self) -> &Subject;

    /// Optional reply subject to which response can be published by a subscriber or consumer.
    fn reply(&self) -> Option<&Subject>;

    /// Optional headers.
    fn headers(&self) -> Option<&HeaderMap>;

    /// Optional Status of the message. Used mostly for internal handling.
    fn status(&self) -> Option<StatusCode>;

    /// Optional status description.
    fn description(&self) -> Option<&str>;

    /// Length of message in bytes.
    fn length(&self) -> usize;

    /// Length of message's payload in bytes.
    fn payload_length(&self) -> usize;

    fn from_head_and_payload(
        head: Head,
        payload: Bytes,
    ) -> Result<(Self, Extensions), FromPartsError>
    where
        Self: Sized;

    fn into_head_and_payload(self) -> (Head, Bytes);
}

impl MessageHead for async_nats::Message {
    fn subject(&self) -> &Subject {
        &self.subject
    }

    fn reply(&self) -> Option<&Subject> {
        self.reply.as_ref()
    }

    fn headers(&self) -> Option<&HeaderMap> {
        self.headers.as_ref()
    }

    fn status(&self) -> Option<StatusCode> {
        self.status
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn length(&self) -> usize {
        self.length
    }

    fn payload_length(&self) -> usize {
        self.payload.len()
    }

    fn from_head_and_payload(
        head: Head,
        payload: Bytes,
    ) -> Result<(Self, Extensions), FromPartsError> {
        let Head {
            subject,
            reply,
            headers,
            status,
            description,
            length,
            extensions,
        } = head;

        Ok((
            Self {
                subject,
                reply,
                payload,
                headers,
                status,
                description,
                length,
            },
            extensions,
        ))
    }

    fn into_head_and_payload(self) -> (Head, Bytes) {
        let Self {
            subject,
            reply,
            payload,
            headers,
            status,
            description,
            length,
        } = self;

        (
            Head {
                subject,
                reply,
                headers,
                status,
                description,
                length,
                extensions: Extensions::new(),
            },
            payload,
        )
    }
}

impl MessageHead for async_nats::jetstream::Message {
    fn subject(&self) -> &Subject {
        &self.message.subject
    }

    fn reply(&self) -> Option<&Subject> {
        self.message.reply.as_ref()
    }

    fn headers(&self) -> Option<&HeaderMap> {
        self.message.headers.as_ref()
    }

    fn status(&self) -> Option<StatusCode> {
        self.message.status
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn length(&self) -> usize {
        self.message.length
    }

    fn payload_length(&self) -> usize {
        self.message.payload.len()
    }

    fn from_head_and_payload(
        head: Head,
        payload: Bytes,
    ) -> Result<(Self, Extensions), FromPartsError> {
        let Head {
            subject,
            reply,
            headers,
            status,
            description,
            length,
            mut extensions,
        } = head;
        let message = async_nats::Message {
            subject,
            reply,
            payload,
            headers,
            status,
            description,
            length,
        };
        let context = extensions
            .remove::<async_nats::jetstream::Context>()
            .ok_or(FromPartsError(
                "jetstream context not found in message head extensions",
            ))?;

        Ok((Self { message, context }, extensions))
    }

    fn into_head_and_payload(self) -> (Head, Bytes) {
        let Self { message, context } = self;
        let async_nats::Message {
            subject,
            reply,
            payload,
            headers,
            status,
            description,
            length,
        } = message;

        let mut extensions = Extensions::new();
        extensions.insert(context);

        (
            Head {
                subject,
                reply,
                headers,
                status,
                description,
                length,
                extensions,
            },
            payload,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Head {
    /// Subject to which message is published to.
    pub subject: Subject,

    /// Optional reply subject to which response can be published by a subscriber or consumer.
    pub reply: Option<Subject>,

    /// Optional headers.
    pub headers: Option<HeaderMap>,

    /// Optional Status of the message. Used mostly for internal handling.
    pub status: Option<StatusCode>,

    /// Optional status description.
    pub description: Option<String>,

    /// Length of message in bytes.
    pub length: usize,

    /// The message's extensions
    pub extensions: Extensions,
}

#[derive(Clone, Debug)]
pub struct HeadRef<'a> {
    /// Subject to which message is published to.
    pub subject: &'a Subject,

    /// Optional reply subject to which response can be published by a subscriber or consumer.
    pub reply: Option<&'a Subject>,

    /// Optional headers.
    pub headers: Option<&'a HeaderMap>,

    /// Optional Status of the message. Used mostly for internal handling.
    pub status: Option<StatusCode>,

    /// Optional status description.
    pub description: Option<&'a str>,

    /// Length of message in bytes.
    pub length: usize,

    /// Length of message's payload in bytes.
    pub payload_length: usize,

    /// The message's extensions
    pub extensions: &'a Extensions,
}
