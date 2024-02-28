use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use tokio::time::Sleep;

pin_project! {
    pub struct ResponseFuture<T> {
        #[pin]
        pub(crate) response: T,
        #[pin]
        pub(crate) sleep: Sleep,
    }
}

impl<F, T> Future for ResponseFuture<F>
where
    F: Future<Output = Result<T, Infallible>>,
{
    type Output = Result<T, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // First, try polling the wait/sleep
        if this.sleep.poll(cx).is_pending() {
            return Poll::Pending;
        }

        // Now poll the response future
        this.response.poll(cx)
    }
}
