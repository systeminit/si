use std::convert::Infallible;

use super::{
    IntoResponse,
    Response,
};

pub trait IntoResponseParts {
    type Error: IntoResponse;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error>;
}

impl<T> IntoResponseParts for Option<T>
where
    T: IntoResponseParts,
{
    type Error = T::Error;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        if let Some(inner) = self {
            inner.into_response_parts(res)
        } else {
            Ok(res)
        }
    }
}

#[derive(Debug)]
pub struct ResponseParts {
    pub(crate) res: Response,
}

impl ResponseParts {}

macro_rules! impl_into_response_parts {
    ( $($ty:ident),* $(,)? ) => {
        #[allow(non_snake_case)]
        impl<$($ty,)*> IntoResponseParts for ($($ty,)*)
        where
            $( $ty: IntoResponseParts, )*
        {
            type Error = Response;

            fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
                let ($($ty,)*) = self;

                $(
                    let res = match $ty.into_response_parts(res) {
                        Ok(res) => res,
                        Err(err) => {
                            return Err(err.into_response());
                        }
                    };
                )*

                Ok(res)
            }
        }
    };
}

all_the_tuples_no_last_special_case!(impl_into_response_parts);

impl IntoResponseParts for () {
    type Error = Infallible;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        Ok(res)
    }
}
