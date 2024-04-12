use std::{
    convert::Infallible,
    fmt,
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
};

use tower::{Layer, Service, ServiceExt};

use crate::{
    extract::FromMessageHead,
    response::{IntoResponse, Response},
    MessageHead,
};

pub struct HandleErrorLayer<F, T> {
    f: F,
    _extractor: PhantomData<fn() -> T>,
}

impl<F, T> HandleErrorLayer<F, T> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            _extractor: PhantomData,
        }
    }
}

impl<F, T> Clone for HandleErrorLayer<F, T>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            _extractor: PhantomData,
        }
    }
}

impl<F, T> fmt::Debug for HandleErrorLayer<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandleErrorLayer")
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

impl<S, F, T> Layer<S> for HandleErrorLayer<F, T>
where
    F: Clone,
{
    type Service = HandleError<S, F, T>;

    fn layer(&self, inner: S) -> Self::Service {
        HandleError::new(inner, self.f.clone())
    }
}

pub struct HandleError<S, F, T> {
    inner: S,
    f: F,
    _extractor: PhantomData<fn() -> T>,
}

impl<S, F, T> HandleError<S, F, T> {
    pub fn new(inner: S, f: F) -> Self {
        Self {
            inner,
            f,
            _extractor: PhantomData,
        }
    }
}

impl<S, F, T> Clone for HandleError<S, F, T>
where
    S: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            f: self.f.clone(),
            _extractor: PhantomData,
        }
    }
}

impl<S, F, E> fmt::Debug for HandleError<S, F, E>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandleError")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

impl<S, R, F, Fut, Res> Service<R> for HandleError<S, F, ()>
where
    S: Service<R> + Clone + Send + 'static,
    S::Response: Send,
    S::Error: Send,
    S::Future: Send,
    F: FnOnce(S::Error) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
    R: MessageHead + Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = future::HandleErrorFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: R) -> Self::Future {
        let f = self.f.clone();

        let clone = self.inner.clone();
        let inner = std::mem::replace(&mut self.inner, clone);

        let future = Box::pin(async move {
            // TODO(fnichol): at the moment the response type `Res` is generic, however we know
            // that by the end we want our response type to be unit (i.e. `()`). Should we shut
            // down the extra generics or allow the final response types to be dropped here?
            // Unclear...
            #[allow(clippy::unit_arg)]
            match inner.oneshot(req).await {
                Ok(_res) => Ok(Response::default()),
                Err(err) => Ok(f(err).await.into_response()),
            }
        });

        future::HandleErrorFuture { future }
    }
}

#[allow(unused_macros)]
macro_rules! impl_service {
    ( $($ty:ident),* $(,)? ) => {
        impl<S, R, F, Res, Fut, $($ty,)*> Service<R>
            for HandleError<S, F, ($($ty,)*)>
        where
            S: Service<R> + Clone + Send + 'static,
            S::Response: IntoResponse + Send,
            S::Error: Send,
            S::Future: Send,
            F: FnOnce($($ty),*, S::Error) -> Fut + Clone + Send + 'static,
            Fut: Future<Output = Res> + Send,
            Res: IntoResponse,
            $( $ty: FromMessageHead<()> + Send,)*
            R: MessageHead + Send + 'static,
        {
            type Response = Response;
            type Error = Infallible;

            type Future = future::HandleErrorFuture;

            fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            #[allow(non_snake_case)]
            fn call(&mut self, req: R) -> Self::Future {
                let f = self.f.clone();

                let clone = self.inner.clone();
                let inner = std::mem::replace(&mut self.inner, clone);

                let future = Box::pin(async move {
                    let (mut parts, body) = req.into_parts();

                    $(
                        let $ty = match $ty::from_message_head(&mut parts, &()).await {
                            Ok(value) => value,
                            Err(rejection) => return Ok(rejection.into_response()),
                        };
                    )*

                    let req = match R::from_parts(parts, body) {
                        Ok(value) => value,
                        Err(rejection) => return Ok(rejection.into_response()),
                    };

                    match inner.oneshot(req).await {
                        Ok(res) => Ok(res.into_response()),
                        Err(err) => Ok(f($($ty),*, err).await.into_response()),
                    }
                });

                future::HandleErrorFuture { future }
            }
        }
    }
}

impl_service!(T1);
impl_service!(T1, T2);
impl_service!(T1, T2, T3);
impl_service!(T1, T2, T3, T4);
impl_service!(T1, T2, T3, T4, T5);
impl_service!(T1, T2, T3, T4, T5, T6);
impl_service!(T1, T2, T3, T4, T5, T6, T7);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

pub mod future {
    use std::{
        convert::Infallible,
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    use pin_project_lite::pin_project;

    use crate::response::Response;

    pin_project! {
        pub struct HandleErrorFuture {
            #[pin]
            pub(super) future: Pin<Box<dyn Future<Output = Result<Response, Infallible>>
                + Send
                + 'static
            >>,
        }
    }

    impl Future for HandleErrorFuture {
        type Output = Result<Response, Infallible>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.project().future.poll(cx)
        }
    }
}
