//! Naxum extractors used by the Rebaser server.
//!
//! TODO(fnichol): these should be extracted to an external crate to be used by other services.
//! These should not be extracted into Naxum itself as the extractors related to custom ways we are
//! using headers.

use nats_std::headers;
use naxum::{
    Head,
    Message,
    MessageHead,
    async_trait,
    composite_rejection,
    define_rejection,
    extract::{
        FromMessage,
        FromMessageHead,
    },
};
use rebaser_core::api_types::{
    ApiVersionsWrapper,
    ApiWrapper,
};
use si_data_nats::Subject;
use telemetry::prelude::*;

define_rejection! {
    #[status_code = 400]
    #[body = "Invalid decoding string as utf8"]
    /// Rejection type for [`HeaderReply`].
    ///
    /// This rejection is used if a found header's subject is not a UTF-8 string.
    pub struct InvalidUtf8Error(Error);
}

/// An extractor which returns an optional reply [`Subject`] from a known header.
///
/// This extractor is useful when requiring a reply on a Jetstream message.
#[derive(Debug)]
pub struct HeaderReply(pub Option<Subject>);

#[async_trait]
impl<S> FromMessageHead<S> for HeaderReply {
    type Rejection = InvalidUtf8Error;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        let maybe_value = head.headers.as_ref().and_then(|headers| {
            headers
                .get(headers::REPLY_INBOX)
                .map(|value| value.to_string())
        });

        match maybe_value {
            Some(value) => Ok(Self(Some(
                Subject::from_utf8(value).map_err(InvalidUtf8Error::from_err)?,
            ))),
            None => Ok(Self(None)),
        }
    }
}

define_rejection! {
    #[status_code = 400]
    #[body = "Headers are required for content info but none was found"]
    /// Rejection type for [`ContentInfo`].
    ///
    /// This rejection is used if no headers were found.
    pub struct HeadersMissing;
}

define_rejection! {
    #[status_code = 400]
    #[body = "Failed to parse content info from headers"]
    /// Rejection type for [`ContentInfo`].
    ///
    /// This rejection is used if info couldn't be parsed from the set of headers.
    pub struct HeadersParseError(Error);
}

composite_rejection! {
    /// Rejection type for [`ContentInfo`].
    ///
    /// Contains one variant for each way the [`ContentInfo`] extractor can fail.
    pub enum ContentInfoRejection {
        HeadersMissing,
        HeadersParseError,
    }
}

/// An extractor for [`rebaser_core::api_types::ContentInfo`].
#[derive(Debug)]
pub struct ContentInfo(pub rebaser_core::api_types::ContentInfo<'static>);

#[async_trait]
impl<S> FromMessageHead<S> for ContentInfo {
    type Rejection = ContentInfoRejection;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = head.headers.as_ref().ok_or(HeadersMissing)?;
        let content_info = rebaser_core::api_types::ContentInfo::try_from(headers)
            .map_err(HeadersParseError::from_err)?;

        Ok(Self(content_info))
    }
}

define_rejection! {
    #[status_code = 400]
    #[body = "failed to deserialize message payload"]
    /// Rejection type for [`ApiTypesNegotiate`].
    ///
    /// This rejection is used if the message fails to deserialize.
    pub struct DeserializeError(Error);
}

define_rejection! {
    #[status_code = 400]
    #[body = "failed to upgrade message to current version"]
    /// Rejection type for [`ApiTypesNegotiate`].
    ///
    /// This rejection is used if the message fails to be upgraded the current known version.
    pub struct MessageUpgradeError(Error);
}

define_rejection! {
    #[status_code = 415]
    #[body = "unsupported content type"]
    /// Rejection type for [`ApiTypesNegotiate`].
    ///
    /// This rejection is used if the content type (i.e. serialization format) is not supported.
    pub struct UnsupportedContentTypeError;
}

define_rejection! {
    #[status_code = 406]
    #[body = "unsupported message type"]
    /// Rejection type for [`ApiTypesNegotiate`].
    ///
    /// This rejection is used if the message type is not supported.
    pub struct UnsupportedMessageTypeError;
}

define_rejection! {
    #[status_code = 406]
    #[body = "unsupported message version"]
    /// Rejection type for [`ApiTypesNegotiate`].
    ///
    /// This rejection is used if the message version is not supported.
    pub struct UnsupportedMessageVersionError;
}

composite_rejection! {
    /// Rejection for [`ApiTypesNegotiate`].
    ///
    /// Contains one variant for each way the [`ApiTypesNegotiate`] extractor can fail.
    pub enum ApiTypesNegotiateRejection {
        ContentInfoRejection,
        DeserializeError,
        MessageUpgradeError,
        UnsupportedContentTypeError,
        UnsupportedMessageTypeError,
        UnsupportedMessageVersionError,
    }
}

/// An extractor which determines the type, versioning, and serialization of a API message.
#[derive(Clone, Copy, Debug, Default)]
#[must_use]
pub struct ApiTypesNegotiate<T>(pub T);

#[async_trait]
impl<T, S, R> FromMessage<S, R> for ApiTypesNegotiate<T>
where
    T: ApiWrapper,
    R: MessageHead + Send + 'static,
    S: Send + Sync,
{
    type Rejection = ApiTypesNegotiateRejection;

    async fn from_message(req: Message<R>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut head, payload) = req.into_parts();
        let ContentInfo(content_info) = ContentInfo::from_message_head(&mut head, state).await?;

        if !T::is_content_type_supported(content_info.content_type.as_str()) {
            return Err(UnsupportedContentTypeError.into());
        }
        if !T::is_message_type_supported(content_info.message_type.as_str()) {
            return Err(UnsupportedMessageTypeError.into());
        }
        if !T::is_message_version_supported(content_info.message_version.as_u64()) {
            return Err(UnsupportedMessageVersionError.into());
        }

        let deserialized_versions = T::from_slice(content_info.content_type.as_str(), &payload)
            .map_err(DeserializeError::from_err)?;
        let current_version = deserialized_versions
            .into_current_version()
            .map_err(MessageUpgradeError::from_err)?;

        Ok(Self(current_version))
    }
}
