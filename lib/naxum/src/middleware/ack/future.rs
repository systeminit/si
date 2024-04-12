use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::future::BoxFuture;
use pin_project_lite::pin_project;
use tokio_util::sync::DropGuard;
use tower::Service;

use crate::response::Response;

pin_project! {
    pub struct ResponseFuture<S>
    where
        S: Service<async_nats::Message>,
    {
        #[pin]
        pub(crate) inner: S::Future,
        #[pin]
        pub(crate) on_success_fut: BoxFuture<'static, ()>,
        #[pin]
        pub(crate) on_failure_fut: BoxFuture<'static, ()>,
        pub(crate) state: State<S::Response, S::Error>,
        pub(crate) shutdown_guard: Option<DropGuard>,
    }
}

#[derive(Clone, Default)]
pub(crate) enum State<T, E> {
    #[default]
    Initial,
    Success(Option<T>),
    Failure(Option<T>),
    Err(Option<E>),
}

impl<S> Future for ResponseFuture<S>
where
    S: Service<async_nats::Message, Response = Response>,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match this.state {
                // Poll the nested service to yield our result
                State::Initial => {
                    let result = futures::ready!(this.inner.as_mut().poll(cx));

                    // Cancel the associated `MaintainProgressTask`. Note that any sufficiently
                    // long-running `on_success`/`on_failure` callbacks will need to manage their
                    // own progress acking--we've only maintained progress acking for the duration
                    // of the inner service's execution time.
                    this.shutdown_guard
                        .take()
                        .expect("extracting shutdown guard value only happens once")
                        .disarm()
                        .cancel();

                    match result {
                        Ok(response) => {
                            if response.status().is_server_error() {
                                // Transition the state to run the failure case
                                *this.state = State::Failure(Some(response));
                            } else {
                                // Transition the state to run the success case
                                *this.state = State::Success(Some(response));
                            }
                        }
                        Err(err) => {
                            // Transition the state to run the failure case
                            *this.state = State::Err(Some(err));
                        }
                    }
                }
                // Poll the on_success future and when ready return the `Ok` type
                State::Success(sucess_response) => {
                    futures::ready!(this.on_success_fut.poll(cx));
                    return Poll::Ready(Ok(sucess_response
                        .take()
                        .expect("extracting owned value only happens once")));
                }
                // Poll the on_failure future and when ready return the `Ok` type
                State::Failure(failure_response) => {
                    futures::ready!(this.on_failure_fut.poll(cx));
                    return Poll::Ready(Ok(failure_response
                        .take()
                        .expect("extracting owned value only happens once")));
                }
                // Poll the on_failure future and when ready return the `Err` type
                State::Err(err) => {
                    futures::ready!(this.on_failure_fut.poll(cx));
                    return Poll::Ready(Err(err
                        .take()
                        .expect("extracting owned value only happens once")));
                }
            }
        }
    }
}
