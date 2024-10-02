use std::time::Instant;

use bytes::Bytes;
use tracing::Span;

use crate::body::inner;

pub struct ResponseBody<B> {
    pub(crate) inner: B,
    pub(crate) _start: Instant,
    pub(crate) _span: Span,
}

impl<B> From<ResponseBody<B>> for Bytes
where
    B: inner::Body,
{
    fn from(value: ResponseBody<B>) -> Self {
        value.inner.into()
    }
}

impl<B> inner::Body for ResponseBody<B> where B: inner::Body {}
