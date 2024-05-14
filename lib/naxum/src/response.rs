mod into_response;

use async_nats::StatusCode;

pub use self::into_response::IntoResponse;

#[derive(Clone, Copy, Debug, Default)]
pub struct Response {
    status: StatusCode,
}

impl Response {
    pub fn server_error() -> Self {
        Self {
            status: StatusCode::from_u16(500).expect("status code is in valid range"),
        }
    }

    pub fn status(self) -> StatusCode {
        self.status
    }
}

pub type Result<T, E = ErrorResponse> = std::result::Result<T, E>;

impl<T> IntoResponse for Result<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(err) => err.0,
        }
    }
}

#[derive(Debug)]
pub struct ErrorResponse(Response);

impl<T> From<T> for ErrorResponse
where
    T: IntoResponse,
{
    fn from(value: T) -> Self {
        #[allow(clippy::unit_arg)]
        Self(value.into_response())
    }
}
