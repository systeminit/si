use acceptable::{
    ContentInfoError,
    HeaderMapParseMessageInfoError,
    NegotiateError,
};
use async_nats::Subject;
use nats_std::header;
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
                .get(header::REPLY_INBOX)
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
    #[body = "Custom error, bad request"]
    /// Rejection type for [`ContentInfo`].
    ///
    /// This rejection is used other errors are found unrelated to parsing NATS headers.
    pub struct CustomError(Error);
}

define_rejection! {
    #[status_code = 400]
    #[body = "Headers are required for content info but none were found"]
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
        CustomError,
        HeadersMissing,
        HeadersParseError,
    }
}

/// An extractor for [`rebaser_core::api_types::ContentInfo`].
#[derive(Debug)]
pub struct ContentInfo(pub acceptable::ContentInfo<'static>);

#[async_trait]
impl<S> FromMessageHead<S> for ContentInfo {
    type Rejection = ContentInfoRejection;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = head.headers.as_ref().ok_or(HeadersMissing)?;
        let content_info =
            acceptable::ContentInfo::try_from(headers).map_err(HeadersParseError::from_err)?;

        Ok(Self(content_info))
    }
}

define_rejection! {
    #[status_code = 400]
    #[body = "failed to deserialize message payload"]
    /// Rejection type for [`Negotiate`].
    ///
    /// This rejection is used if the message fails to deserialize.
    pub struct DeserializeError(Error);
}

define_rejection! {
    #[status_code = 400]
    #[body = "failed to upgrade message to current version"]
    /// Rejection type for [`Negotiate`].
    ///
    /// This rejection is used if the message fails to be upgraded the current known version.
    pub struct MessageUpgradeError(Error);
}

define_rejection! {
    #[status_code = 415]
    #[body = "unsupported content type"]
    /// Rejection type for [`Negotiate`].
    ///
    /// This rejection is used if the content type (i.e. serialization format) is not supported.
    pub struct UnsupportedContentTypeError;
}

define_rejection! {
    #[status_code = 406]
    #[body = "unsupported message type"]
    /// Rejection type for [`Negotiate`].
    ///
    /// This rejection is used if the message type is not supported.
    pub struct UnsupportedMessageTypeError;
}

define_rejection! {
    #[status_code = 406]
    #[body = "unsupported message version"]
    /// Rejection type for [`Negotiate`].
    ///
    /// This rejection is used if the message version is not supported.
    pub struct UnsupportedMessageVersionError;
}

composite_rejection! {
    /// Rejection for [`Negotiate`].
    ///
    /// Contains one variant for each way the [`Negotiate`] extractor can fail.
    pub enum NegotiateRejection {
        ContentInfoRejection,
        DeserializeError,
        MessageUpgradeError,
        UnsupportedContentTypeError,
        UnsupportedMessageTypeError,
        UnsupportedMessageVersionError,
    }
}

impl From<NegotiateError> for NegotiateRejection {
    fn from(value: NegotiateError) -> Self {
        match value {
            NegotiateError::ContentInfo(ContentInfoError(err)) => {
                match err
                    .downcast_ref::<HeaderMapParseMessageInfoError>()
                    .as_ref()
                {
                    // Error is from parsing NATS headers
                    Some(&HeaderMapParseMessageInfoError::MissingHeader(_))
                    | Some(&HeaderMapParseMessageInfoError::MissingHeaders) => {
                        ContentInfoRejection::HeadersMissing(HeadersMissing).into()
                    }
                    // Error is from parsing NATS headers
                    Some(&HeaderMapParseMessageInfoError::ParseVersion(err)) => {
                        ContentInfoRejection::HeadersParseError(HeadersParseError::from_err(
                            err.clone(),
                        ))
                        .into()
                    }
                    // Any other boxed error
                    None => ContentInfoRejection::CustomError(CustomError::from_err(err)).into(),
                }
            }
            NegotiateError::Deserialize(acceptable::DeserializeError::UnsupportedContentType(
                _,
            ))
            | NegotiateError::UnsupportedContentType(_) => UnsupportedContentTypeError.into(),
            NegotiateError::Deserialize(acceptable::DeserializeError::Deserialize(err)) => {
                DeserializeError::from_err(err).into()
            }
            NegotiateError::Deserialize(acceptable::DeserializeError::Upgrade(err)) => {
                MessageUpgradeError::from_err(err).into()
            }
            NegotiateError::UnsupportedMessageType(_) => UnsupportedMessageTypeError.into(),
            NegotiateError::UnsupportedMessageVersion(_) => UnsupportedMessageVersionError.into(),
        }
    }
}

/// An extractor which determines the type, versioning, and serialization of a API message.
#[derive(Clone, Copy, Debug, Default)]
#[must_use]
pub struct Negotiate<T>(pub T);

#[async_trait]
impl<T, S, R> FromMessage<S, R> for Negotiate<T>
where
    T: acceptable::Negotiate,
    R: MessageHead + Send + 'static,
    S: Send + Sync,
{
    type Rejection = NegotiateRejection;

    async fn from_message(req: Message<R>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut head, payload) = req.into_parts();
        let ContentInfo(content_info) = ContentInfo::from_message_head(&mut head, state).await?;

        Ok(Self(
            T::negotiate(&content_info, &payload).map_err(Self::Rejection::from)?,
        ))
    }
}
