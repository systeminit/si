use std::convert::Infallible;

use async_trait::async_trait;

use crate::{
    message::Head,
    response::{IntoResponse, Response},
    MessageHead,
};

use super::{FromMessage, FromMessageHead};

#[async_trait]
impl<S> FromMessageHead<S> for ()
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_message_head(_head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(())
    }
}

macro_rules! impl_from_message {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[async_trait]
        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<S, $($ty,)* $last> FromMessageHead<S> for ($($ty,)* $last,)
        where
            $( $ty: FromMessageHead<S> + Send, )*
            $last: FromMessageHead<S> + Send,
            S: Send + Sync,
        {
            type Rejection = Response;

            async fn from_message_head(head: &mut Head, state: &S) -> Result<Self, Self::Rejection> {
                $(
                    let $ty = $ty::from_message_head(head, state)
                        .await
                        .map_err(|err| err.into_response())?;
                )*
                let $last = $last::from_message_head(head, state)
                    .await
                    .map_err(|err| err.into_response())?;

                Ok(($($ty,)* $last,))
            }
        }

        // This impl must not be generic over M, otherwise it would conflict with the blanket
        // implementation of `FromMessage<S, Mut>` for `T: FromMessageHead<S>`.
        #[async_trait]
        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<S, R, $($ty,)* $last> FromMessage<S, R> for ($($ty,)* $last,)
        where
            $( $ty: FromMessageHead<S> + Send, )*
            $last: FromMessage<S, R> + Send,
            S: Send + Sync,
            R: MessageHead + Send + 'static,
        {
            type Rejection = Response;

            async fn from_message(req: R, state: &S) -> Result<Self, Self::Rejection> {
                let (mut head, body) = req.into_parts();

                $(
                    let $ty = $ty::from_message_head(&mut head, state).await.map_err(|err| err.into_response())?;
                )*

                let req = R::from_parts(head, body).map_err(|err| err.into_response())?;

                let $last = $last::from_message(req, state).await.map_err(|err| err.into_response())?;

                Ok(($($ty,)* $last,))
            }
        }
    };
}

all_the_tuples!(impl_from_message);
