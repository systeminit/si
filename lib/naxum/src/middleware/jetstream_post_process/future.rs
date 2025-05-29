use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use async_nats::jetstream;
use futures::future::BoxFuture;
use pin_project_lite::pin_project;
use tower::Service;

use crate::{
    message::Message,
    response::Response,
};

pin_project! {
    pub struct ResponseFuture<S>
    where
        S: Service<Message<jetstream::Message>>,
    {
        #[pin]
        pub(crate) inner: S::Future,
        #[pin]
        pub(crate) on_success_fut: BoxFuture<'static, ()>,
        #[pin]
        pub(crate) on_failure_fut: BoxFuture<'static, ()>,
        pub(crate) state: State<S::Response, S::Error>,
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
    S: Service<Message<jetstream::Message>, Response = Response>,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match this.state {
                // Poll the nested service to yield our result
                State::Initial => {
                    let result = futures::ready!(this.inner.as_mut().poll(cx));

                    match result {
                        Ok(response) => {
                            if response.status().is_success() {
                                // Transition the state to run the success case
                                *this.state = State::Success(Some(response));
                            } else {
                                // Transition the state to run the failure case
                                *this.state = State::Failure(Some(response));
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
