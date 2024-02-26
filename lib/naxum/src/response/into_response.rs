use core::fmt;
use std::{borrow::Cow, convert::Infallible};

use async_nats::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use bytes::{buf::Chain, Buf, Bytes, BytesMut};

use super::Response;

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        self
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

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {}
}

impl IntoResponse for String {
    fn into_response(self) -> Response {}
}

impl IntoResponse for Box<str> {
    fn into_response(self) -> Response {}
}

impl IntoResponse for Cow<'static, str> {
    fn into_response(self) -> Response {}
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {}
}

impl IntoResponse for BytesMut {
    fn into_response(self) -> Response {}
}

impl<T, U> IntoResponse for Chain<T, U>
where
    T: Buf + Unpin + Send + 'static,
    U: Buf + Unpin + Send + 'static,
{
    fn into_response(self) -> Response {}
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {}
}

impl<const N: usize> IntoResponse for [u8; N] {
    fn into_response(self) -> Response {}
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {}
}

impl IntoResponse for Box<[u8]> {
    fn into_response(self) -> Response {}
}

impl IntoResponse for Cow<'static, [u8]> {
    fn into_response(self) -> Response {}
}

impl<R> IntoResponse for (StatusCode, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {}
}

impl IntoResponse for HeaderMap {
    fn into_response(self) -> Response {}
}

impl<K, V, const N: usize> IntoResponse for [(K, V); N]
where
    K: TryInto<HeaderName>,
    K::Error: fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: fmt::Display,
{
    fn into_response(self) -> Response {}
}

impl<R> IntoResponse for (R,)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {}
}
