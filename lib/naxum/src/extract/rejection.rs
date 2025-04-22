use std::{
    error,
    fmt,
};

use crate::{
    BoxError,
    Error,
    composite_rejection,
    define_rejection,
    response::{
        IntoResponse,
        Response,
    },
};

#[derive(Debug)]
pub struct InvalidUtf8(Error);

impl InvalidUtf8 {
    pub(crate) fn from_err<E>(err: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self(Error::new(err))
    }
}

impl IntoResponse for InvalidUtf8 {
    fn into_response(self) -> Response {
        // TODO: log rejection
        Response::default_internal_server_error()
    }
}

impl fmt::Display for InvalidUtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "message payload didn't contain valid UTF-8: {:?}",
            self.0
        )
    }
}

impl error::Error for InvalidUtf8 {}

#[derive(Debug)]
pub enum StringRejection {
    InvalidUtf8(InvalidUtf8),
}

impl IntoResponse for StringRejection {
    fn into_response(self) -> crate::response::Response {
        match self {
            StringRejection::InvalidUtf8(inner) => inner.into_response(),
        }
    }
}

impl From<InvalidUtf8> for StringRejection {
    fn from(value: InvalidUtf8) -> Self {
        Self::InvalidUtf8(value)
    }
}

impl fmt::Display for StringRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8(inner) => write!(f, "{inner}"),
        }
    }
}

impl error::Error for StringRejection {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(inner) => inner.source(),
        }
    }
}

define_rejection! {
    #[status_code = 400]
    #[body = "Reply was required for message but none was found"]
    /// Rejection type for [`RequiredReply`].
    ///
    /// This rejection is used if a reply was expected on a message but one was not provided.
    pub struct NoReplyRejection;
}

define_rejection! {
    #[status_code = 422]
    #[body = "Failed to deserialize the JSON body into the target type"]
    /// Rejection type for [`Json`].
    ///
    /// This rejection is used if the message body is syntactically valid JSON but couldn't be
    /// deserialized into the target type.
    pub struct JsonDataError(Error);
}

define_rejection! {
    #[status_code = 400]
    #[body = "Failed to parse the message body as JSON"]
    /// Rejection type for [`Json`].
    ///
    /// This rejection is used if the message body didn't contain syntactically valid JSON.
    pub struct JsonSyntaxError(Error);
}

composite_rejection! {
    /// Rejection type for [`Json`].
    ///
    /// Contains one vaiant for each way the [`Json`] extractor can fail.
    pub enum JsonRejection {
        JsonDataError,
        JsonSyntaxError,
        // MissingJsonContentType,
    }
}

define_rejection! {
    #[status_code = 500]
    #[body = "No matched subject found"]
    /// Rejection type for [`MatchedSubject`](super::MatchedSubject).
    ///
    /// This rejection is used if no matched subject could be found.
    pub struct MatchedSubjectMissing;
}

composite_rejection! {
    /// Rejection type for [`MatchedSubject`](super::MatchedSubject).
    ///
    /// Contains one vaiant for each way the extractor can fail.
    pub enum MatchedSubjectRejection {
        MatchedSubjectMissing,
    }
}
