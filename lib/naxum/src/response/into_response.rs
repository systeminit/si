use core::fmt;
use std::{borrow::Cow, convert::Infallible};

use async_nats::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use bytes::{Bytes, BytesMut};

use crate::body::{self, Body};

use super::{
    IntoResponseParts, Response, ResponseParts,
    inner::{self, Parts},
};

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        let mut response = ().into_response();
        *response.status_mut() = self;
        response
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Body::empty().into_response()
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

impl<B> IntoResponse for Response<B>
where
    B: body::inner::Body,
{
    fn into_response(self) -> Response {
        self.map(Body::new)
    }
}

impl IntoResponse for Parts {
    fn into_response(self) -> Response {
        Response::from_parts(self, Body::empty())
    }
}

impl IntoResponse for Body {
    fn into_response(self) -> Response {
        Response::new(self)
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Cow::Borrowed(self).into_response()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Cow::<'static, str>::Owned(self).into_response()
    }
}

impl IntoResponse for Box<str> {
    fn into_response(self) -> Response {
        String::from(self).into_response()
    }
}

impl IntoResponse for Cow<'static, str> {
    fn into_response(self) -> Response {
        Body::from(self).into_response()
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        Body::from(self).into_response()
    }
}

impl IntoResponse for BytesMut {
    fn into_response(self) -> Response {
        self.freeze().into_response()
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        Cow::Borrowed(self).into_response()
    }
}

impl<const N: usize> IntoResponse for &'static [u8; N] {
    fn into_response(self) -> Response {
        self.as_slice().into_response()
    }
}

impl<const N: usize> IntoResponse for [u8; N] {
    fn into_response(self) -> Response {
        self.to_vec().into_response()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        Cow::<'static, [u8]>::Owned(self).into_response()
    }
}

impl IntoResponse for Box<[u8]> {
    fn into_response(self) -> Response {
        Vec::from(self).into_response()
    }
}

impl IntoResponse for Cow<'static, [u8]> {
    fn into_response(self) -> Response {
        Body::from(self).into_response()
    }
}

impl<R> IntoResponse for (StatusCode, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut response = self.1.into_response();
        *response.status_mut() = self.0;
        response
    }
}

impl IntoResponse for HeaderMap {
    fn into_response(self) -> Response {
        ().into_response()
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

impl<R> IntoResponse for (Parts, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        let (parts, res) = self;
        (parts.status, res).into_response()
    }
}

impl<R> IntoResponse for (inner::Response<()>, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        let (template, res) = self;
        let (parts, ()) = template.into_parts();
        (parts, res).into_response()
    }
}

impl<R> IntoResponse for (R,)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        let (res,) = self;
        res.into_response()
    }
}

macro_rules! impl_into_response {
    ( $($ty:ident),* $(,)? ) => {
        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for ($($ty),*, R)
        where
            $( $ty: IntoResponseParts, )*
            R: IntoResponse,
        {
            fn into_response(self) -> Response {
                let ($($ty),*, res) = self;

                let res = res.into_response();
                let parts = ResponseParts { res };

                $(
                    let parts = match $ty.into_response_parts(parts) {
                        Ok(parts) => parts,
                        Err(err) => {
                            return err.into_response();
                        }
                    };
                )*

                parts.res
            }
        }

        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for (StatusCode, $($ty),*, R)
        where
            $( $ty: IntoResponseParts, )*
            R: IntoResponse,
        {
            fn into_response(self) -> Response {
                let (status, $($ty),*, res) = self;

                let res = res.into_response();
                let parts = ResponseParts { res };

                $(
                    let parts = match $ty.into_response_parts(parts) {
                        Ok(parts) => parts,
                        Err(err) => {
                            return err.into_response();
                        }
                    };
                )*

                (status, parts.res).into_response()
            }
        }

        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for (Parts, $($ty),*, R)
        where
            $( $ty: IntoResponseParts, )*
            R: IntoResponse,
        {
            fn into_response(self) -> Response {
                let (outer_parts, $($ty),*, res) = self;

                let res = res.into_response();
                let parts = ResponseParts { res };

                $(
                    let parts = match $ty.into_response_parts(parts) {
                        Ok(parts) => parts,
                        Err(err) => {
                            return err.into_response();
                        }
                    };
                )*

                (outer_parts, parts.res).into_response()
            }
        }

        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for (inner::Response<()>, $($ty),*, R)
        where
            $( $ty: IntoResponseParts, )*
            R: IntoResponse,
        {
            fn into_response(self) -> Response {
                let (template, $($ty),*, res) = self;
                let (parts, ()) = template.into_parts();
                (parts, $($ty),*, res).into_response()
            }
        }
    };
}

all_the_tuples_no_last_special_case!(impl_into_response);
