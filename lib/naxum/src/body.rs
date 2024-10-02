use std::borrow::Cow;

use bytes::Bytes;

pub mod inner;

#[derive(Debug)]
pub struct Body(Bytes);

impl Body {
    pub fn new<B>(body: B) -> Self
    where
        B: inner::Body,
    {
        Self(body.into())
    }

    pub const fn empty() -> Self {
        Self(Bytes::new())
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Self::empty()
    }
}

impl From<&'static [u8]> for Body {
    fn from(value: &'static [u8]) -> Self {
        Self(Bytes::from(value))
    }
}

impl From<Cow<'static, [u8]>> for Body {
    fn from(value: Cow<'static, [u8]>) -> Self {
        match value {
            Cow::Borrowed(value) => Self(Bytes::from(value)),
            Cow::Owned(value) => Self(Bytes::from(value)),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Self(Bytes::from(value))
    }
}

impl From<&'static str> for Body {
    fn from(value: &'static str) -> Self {
        Self(Bytes::from(value))
    }
}

impl From<Cow<'static, str>> for Body {
    fn from(value: Cow<'static, str>) -> Self {
        match value {
            Cow::Borrowed(value) => Self(Bytes::from(value)),
            Cow::Owned(value) => Self(Bytes::from(value)),
        }
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Self(Bytes::from(value))
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl From<Body> for Bytes {
    fn from(value: Body) -> Self {
        value.0
    }
}

impl inner::Body for Body {}
