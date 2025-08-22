use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
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
    Head,
    message::Message,
    middleware::jetstream_post_process::{
        Info,
        OnFailure,
        OnSuccess,
    },
    response::Response,
};

pin_project! {
    pub struct ResponseFuture<S, OnSuccessT, OnFailureT>
    where
        S: Service<Message<jetstream::Message>>,
    {
        #[pin]
        pub(crate) inner: S::Future,
        pub(crate) on_success: OnSuccessT,
        pub(crate) on_failure: OnFailureT,
        pub(crate) state: State<S::Response, S::Error>,
    }
}

pub(crate) enum State<T, E> {
    Initial(Option<(Arc<Head>, Arc<Info>)>),
    Success(Option<T>, BoxFuture<'static, ()>),
    Failure(Option<T>, BoxFuture<'static, ()>),
    Err(Option<E>, BoxFuture<'static, ()>),
}

impl<S, OnSuccessT, OnFailureT> Future for ResponseFuture<S, OnSuccessT, OnFailureT>
where
    S: Service<Message<jetstream::Message>, Response = Response>,
    OnSuccessT: OnSuccess,
    OnFailureT: OnFailure,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match this.state {
                // Poll the nested service to yield our result
                State::Initial(args) => {
                    let result = futures::ready!(this.inner.as_mut().poll(cx));

                    match result {
                        Ok(response) => {
                            if response.status().is_success() {
                                let (head, info) = args
                                    .take()
                                    .expect("extracting owned value only happens once");
                                let status = response.status();

                                // Transition the state to run the success case
                                *this.state = State::Success(
                                    Some(response),
                                    this.on_success.call(head, info, status),
                                );
                            } else {
                                let (head, info) = args
                                    .take()
                                    .expect("extracting owned value only happens once");
                                let status = Some(response.status());

                                // Transition the state to run the failure case
                                *this.state = State::Failure(
                                    Some(response),
                                    this.on_failure.call(head, info, status),
                                );
                            }
                        }
                        Err(err) => {
                            let (head, info) = args
                                .take()
                                .expect("extracting owned value only happens once");
                            let status = None;

                            // Transition the state to run the failure case
                            *this.state =
                                State::Err(Some(err), this.on_failure.call(head, info, status));
                        }
                    }
                }
                // Poll the on_success future and when ready return the `Ok` type
                State::Success(sucess_response, on_success_fut) => {
                    futures::ready!(on_success_fut.as_mut().poll(cx));
                    return Poll::Ready(Ok(sucess_response
                        .take()
                        .expect("extracting owned value only happens once")));
                }
                // Poll the on_failure future and when ready return the `Ok` type
                State::Failure(failure_response, on_failure_fut) => {
                    futures::ready!(on_failure_fut.as_mut().poll(cx));
                    return Poll::Ready(Ok(failure_response
                        .take()
                        .expect("extracting owned value only happens once")));
                }
                // Poll the on_failure future and when ready return the `Err` type
                State::Err(err, on_failure_fut) => {
                    futures::ready!(on_failure_fut.as_mut().poll(cx));
                    return Poll::Ready(Err(err
                        .take()
                        .expect("extracting owned value only happens once")));
                }
            }
        }
    }
}
