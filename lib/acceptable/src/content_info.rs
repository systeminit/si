use std::{
    borrow::Cow,
    fmt,
    num::ParseIntError,
    str::FromStr,
};

use crate::container::Container;
#[cfg(feature = "nats-headers")]
use crate::error::HeaderMapParseMessageInfoError;

/// Content metadata information for a message.
#[derive(Clone, Debug)]
pub struct ContentInfo<'a> {
    /// The type of the content in the message.
    pub content_type: ContentType<'a>,
    /// The type of the message.
    pub message_type: MessageType,
    /// The version of the message.
    pub message_version: MessageVersion,
}

impl<T> From<&T> for ContentInfo<'static>
where
    T: Container,
{
    fn from(_value: &T) -> Self {
        Self {
            content_type: T::default_content_type().into(),
            message_type: T::message_type().into(),
            message_version: T::message_version().into(),
        }
    }
}

#[cfg(feature = "nats-headers")]
impl ContentInfo<'_> {
    /// Injects the content information into NATS message headers.
    pub fn inject_into_headers(&self, headers: &mut async_nats::HeaderMap) {
        headers.insert(nats_std::header::CONTENT_TYPE, self.content_type.as_str());
        headers.insert(nats_std::header::MESSAGE_TYPE, self.message_type.as_str());
        headers.insert(
            nats_std::header::MESSAGE_VERSION,
            self.message_version.to_string(),
        );
    }
}

#[cfg(feature = "nats-headers")]
impl TryFrom<&async_nats::HeaderMap> for ContentInfo<'_> {
    type Error = HeaderMapParseMessageInfoError;

    fn try_from(map: &async_nats::HeaderMap) -> Result<Self, Self::Error> {
        let content_type = ContentType::from(map.get(nats_std::header::CONTENT_TYPE).ok_or(
            HeaderMapParseMessageInfoError::MissingHeader(nats_std::header::CONTENT_TYPE),
        )?);
        let message_type = MessageType::from(map.get(nats_std::header::MESSAGE_TYPE).ok_or(
            HeaderMapParseMessageInfoError::MissingHeader(nats_std::header::MESSAGE_TYPE),
        )?);
        let message_version =
            MessageVersion::try_from(map.get(nats_std::header::MESSAGE_VERSION).ok_or(
                HeaderMapParseMessageInfoError::MissingHeader(nats_std::header::MESSAGE_VERSION),
            )?)
            .map_err(HeaderMapParseMessageInfoError::ParseVersion)?;

        Ok(Self {
            content_type,
            message_type,
            message_version,
        })
    }
}

#[cfg(feature = "nats-headers")]
impl TryFrom<Option<&async_nats::HeaderMap>> for ContentInfo<'_> {
    type Error = HeaderMapParseMessageInfoError;

    fn try_from(value: Option<&async_nats::HeaderMap>) -> Result<Self, Self::Error> {
        let headers = value.ok_or(Self::Error::MissingHeaders)?;
        Self::try_from(headers)
    }
}

/// The content type of a message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentType<'a>(Cow<'a, str>);

impl ContentType<'_> {
    /// Returns the interior data as a borrowed string.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for ContentType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for ContentType<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for ContentType<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

#[cfg(feature = "nats-headers")]
impl From<&async_nats::HeaderValue> for ContentType<'_> {
    fn from(value: &async_nats::HeaderValue) -> Self {
        Self(Cow::Owned(value.as_str().to_string()))
    }
}

/// The type of a message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageType(String);

impl MessageType {
    /// Returns the interior data as a borrowed string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for MessageType {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for MessageType {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[cfg(feature = "nats-headers")]
impl From<&async_nats::HeaderValue> for MessageType {
    fn from(value: &async_nats::HeaderValue) -> Self {
        Self::from(value.as_str())
    }
}

/// The version of a message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageVersion(u64);

impl MessageVersion {
    /// Returns the interior data as an unsigned integer.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for MessageVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u64> for MessageVersion {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl FromStr for MessageVersion {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).map(Self)
    }
}

#[cfg(feature = "nats-headers")]
impl TryFrom<&async_nats::HeaderValue> for MessageVersion {
    type Error = ParseIntError;

    fn try_from(value: &async_nats::HeaderValue) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}
