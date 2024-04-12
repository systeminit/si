use core::fmt;
use std::{borrow::Cow, convert::Infallible};

use async_nats::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use bytes::{buf::Chain, Buf, Bytes, BytesMut};

use super::Response;

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response { status: self }
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Response {
        match self {}
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for Box<str> {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for Cow<'static, str> {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for BytesMut {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl<T, U> IntoResponse for Chain<T, U>
where
    T: Buf + Unpin + Send + 'static,
    U: Buf + Unpin + Send + 'static,
{
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl<const N: usize> IntoResponse for [u8; N] {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for Box<[u8]> {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for Cow<'static, [u8]> {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl<R> IntoResponse for (StatusCode, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        Response { status: self.0 }
    }
}

impl IntoResponse for HeaderMap {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl<K, V, const N: usize> IntoResponse for [(K, V); N]
where
    K: TryInto<HeaderName>,
    K::Error: fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: fmt::Display,
{
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl<R> IntoResponse for (R,)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        Response::default()
    }
}
