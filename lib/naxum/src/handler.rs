use core::fmt;
use std::{
    convert::Infallible,
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
};

use tower::{Layer, Service, ServiceExt};

use crate::{
    extract::{FromMessage, FromMessageHead},
    make_service::IntoMakeService,
    response::{IntoResponse, Response},
    MessageHead,
};

pub mod future;
mod service;

pub use self::service::HandlerService;

pub trait Handler<T, S, R>: Clone + Send + Sized + 'static
where
    R: MessageHead,
{
    type Future: Future<Output = Response> + Send + 'static;

    fn call(self, req: R, state: S) -> Self::Future;

    fn layer<L>(self, layer: L) -> Layered<L, Self, T, S, R>
    where
        L: Layer<HandlerService<Self, T, S, R>> + Clone,
        L::Service: Service<R>,
    {
        Layered {
            layer,
            handler: self,
            _marker: PhantomData,
            _request_marker: PhantomData,
        }
    }

    fn with_state(self, state: S) -> HandlerService<Self, T, S, R> {
        HandlerService::new(self, state)
    }
}

impl<F, Fut, Res, S, R> Handler<((),), S, R> for F
where
    F: FnOnce() -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
    R: MessageHead,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, _req: R, _state: S) -> Self::Future {
        Box::pin(async move { self().await.into_response() })
    }
}

macro_rules! impl_handler {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<F, Fut, S, R, Res, M, $($ty,)* $last> Handler<(M, $($ty,)* $last,), S, R> for F
        where
            F: FnOnce($($ty,)* $last,) -> Fut + Clone + Send + 'static,
            Fut: Future<Output = Res> + Send,
            S: Send + Sync + 'static,
            R: MessageHead + Send + 'static,
            Res: IntoResponse,
            $( $ty: FromMessageHead<S> + Send, )*
            $last: FromMessage<S, R, M> + Send,
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            fn call(self, req: R, state: S) -> Self::Future {
                Box::pin(async move {
                    let (mut parts, body) = req.into_parts();
                    let state = &state;

                    $(
                        let $ty = match $ty::from_message_head(&mut parts, state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*

                    let req = match R::from_parts(parts, body) {
                        Ok(value) => value,
                        Err(rejection) => return rejection.into_response(),
                    };

                    let $last = match $last::from_message(req, state).await {
                        Ok(value) => value,
                        Err(rejection) => return rejection.into_response(),
                    };

                    let res = self($($ty,)* $last,).await;

                    res.into_response()
                })
            }
        }
    };
}

all_the_tuples!(impl_handler);

mod private {
    // Marker type for `impl<T: IntoResponse> Handler for T`
    #[allow(missing_debug_implementations)]
    pub enum IntoResponseHandler {}
}

impl<T, S, R> Handler<private::IntoResponseHandler, S, R> for T
where
    T: IntoResponse + Clone + Send + 'static,
    R: MessageHead,
{
    type Future = Ready<Response>;

    fn call(self, _req: R, _state: S) -> Self::Future {
        #[allow(clippy::unit_arg)]
        ready(self.into_response())
    }
}

pub struct Layered<L, H, T, S, R> {
    layer: L,
    handler: H,
    _marker: PhantomData<fn() -> (T, S)>,
    _request_marker: PhantomData<R>,
}

impl<L, H, T, S, R> fmt::Debug for Layered<L, H, T, S, R>
where
    L: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layered")
            .field("layer", &self.layer)
            .finish_non_exhaustive()
    }
}

impl<L, H, T, S, R> Clone for Layered<L, H, T, S, R>
where
    L: Clone,
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            layer: self.layer.clone(),
            handler: self.handler.clone(),
            _marker: PhantomData,
            _request_marker: PhantomData,
        }
    }
}

impl<L, H, T, S, R> Handler<T, S, R> for Layered<L, H, T, S, R>
where
    L: Layer<HandlerService<H, T, S, R>> + Clone + Send + 'static,
    H: Handler<T, S, R>,
    L::Service: Service<R, Error = Infallible> + Clone + Send + 'static,
    <L::Service as Service<R>>::Response: IntoResponse,
    <L::Service as Service<R>>::Future: Send,
    T: 'static,
    S: 'static,
    R: MessageHead + Send + 'static,
{
    type Future = future::LayeredFuture<L::Service, R>;

    fn call(self, req: R, state: S) -> Self::Future {
        use futures::future::{FutureExt, Map};

        let svc = self.handler.with_state(state);
        let svc = self.layer.layer(svc);

        #[allow(clippy::type_complexity)]
        let future: Map<
            _,
            fn(
                Result<<L::Service as Service<R>>::Response, <L::Service as Service<R>>::Error>,
            ) -> _,
        > = svc.oneshot(req).map(|result| match result {
            Ok(res) => res.into_response(),
            Err(err) => match err {},
        });

        future::LayeredFuture::new(future)
    }
}

pub trait HandlerWithoutStateExt<T, R>: Handler<T, (), R>
where
    R: MessageHead,
{
    fn into_service(self) -> HandlerService<Self, T, (), R>;

    fn into_make_service(self) -> IntoMakeService<HandlerService<Self, T, (), R>>;
}

impl<H, T, R> HandlerWithoutStateExt<T, R> for H
where
    H: Handler<T, (), R>,
    R: MessageHead,
{
    fn into_service(self) -> HandlerService<Self, T, (), R> {
        self.with_state(())
    }

    fn into_make_service(self) -> IntoMakeService<HandlerService<Self, T, (), R>> {
        self.into_service().into_make_service()
    }
}
