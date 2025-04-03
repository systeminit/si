//! Naxum extractors used by the Edda server.
//!
//! TODO(fnichol): these should be extracted to an external crate to be used by other services.
//! These should not be extracted into Naxum itself as the extractors related to custom ways we are
//! using headers.

use bytes::Bytes;
use edda_core::{
    api_types::{
        rebuild_request::RebuildRequest, update_request::UpdateRequest, ApiVersionsWrapper,
        ApiWrapper,
    },
    nats,
};
use naxum::{
    async_trait, composite_rejection, define_rejection,
    extract::{FromMessage, FromMessageHead},
    Head, Message, MessageHead,
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
                .get(nats::NATS_HEADER_REPLY_INBOX_NAME)
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

/// An extractor for [`edda_core::api_types::ContentInfo`].
#[derive(Debug)]
pub struct ContentInfo(pub edda_core::api_types::ContentInfo<'static>);

#[async_trait]
impl<S> FromMessageHead<S> for ContentInfo {
    type Rejection = ContentInfoRejection;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = head.headers.as_ref().ok_or(HeadersMissing)?;
        let content_info = edda_core::api_types::ContentInfo::try_from(headers)
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EddaRequestKind {
    Update(UpdateRequest),
    Rebuild(RebuildRequest),
}

/// An extractor which determines the type, versioning, and serialization of a API message for edda.
#[derive(Clone, Debug)]
#[must_use]
pub struct ApiTypesNegotiate(pub EddaRequestKind);

#[async_trait]
impl<S, R> FromMessage<S, R> for ApiTypesNegotiate
where
    R: MessageHead + Send + 'static,
    S: Send + Sync,
{
    type Rejection = ApiTypesNegotiateRejection;

    async fn from_message(req: Message<R>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut head, payload) = req.into_parts();
        let ContentInfo(content_info) = ContentInfo::from_message_head(&mut head, state).await?;

        let api_type = match content_info.message_type.as_str() {
            <UpdateRequest as ApiWrapper>::MESSAGE_TYPE => {
                EddaRequestKind::Update(negotiate(content_info, &payload).await?)
            }
            <RebuildRequest as ApiWrapper>::MESSAGE_TYPE => {
                EddaRequestKind::Rebuild(negotiate(content_info, &payload).await?)
            }
            _ => return Err(UnsupportedContentTypeError.into()),
        };

        Ok(Self(api_type))
    }
}

async fn negotiate<T>(
    content_info: edda_core::api_types::ContentInfo<'_>,
    payload: &Bytes,
) -> Result<T, ApiTypesNegotiateRejection>
where
    T: ApiWrapper,
{
    if !T::is_content_type_supported(content_info.content_type.as_str()) {
        return Err(UnsupportedContentTypeError.into());
    }
    if !T::is_message_type_supported(content_info.message_type.as_str()) {
        return Err(UnsupportedMessageTypeError.into());
    }
    if !T::is_message_version_supported(content_info.message_version.as_u64()) {
        return Err(UnsupportedMessageVersionError.into());
    }

    let deserialized_versions = T::from_slice(content_info.content_type.as_str(), payload)
        .map_err(DeserializeError::from_err)?;
    let current_version = deserialized_versions
        .into_current_version()
        .map_err(MessageUpgradeError::from_err)?;

    Ok(current_version)
}
