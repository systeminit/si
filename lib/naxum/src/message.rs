use std::{error, fmt};

use async_nats::{HeaderMap, StatusCode, Subject};
use bytes::Bytes;

mod extensions;

pub use extensions::Extensions;

use crate::response::{IntoResponse, Response};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FromPartsError(&'static str);

impl fmt::Display for FromPartsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to build message from parts: {}", self.0)
    }
}

impl error::Error for FromPartsError {}

impl IntoResponse for FromPartsError {
    fn into_response(self) -> Response {}
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

    /// Length of message in bytes.
    fn length(&self) -> usize;

    fn from_parts(head: Head, payload: Bytes) -> Result<Self, FromPartsError>
    where
        Self: Sized;

    fn into_parts(self) -> (Head, Bytes);
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

    fn length(&self) -> usize {
        self.length
    }

    fn from_parts(head: Head, payload: Bytes) -> Result<Self, FromPartsError> {
        let Head {
            subject,
            reply,
            headers,
            status,
            description,
            length,
            extensions: _,
        } = head;
        Ok(Self {
            subject,
            reply,
            payload,
            headers,
            status,
            description,
            length,
        })
    }

    fn into_parts(self) -> (Head, Bytes) {
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

    fn length(&self) -> usize {
        self.message.length
    }

    fn from_parts(head: Head, payload: Bytes) -> Result<Self, FromPartsError> {
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
        Ok(Self { message, context })
    }

    fn into_parts(self) -> (Head, Bytes) {
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
