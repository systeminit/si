use std::{error, fmt};

use crate::{
    response::{IntoResponse, Response},
    BoxError, Error,
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
        Response::server_error()
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
