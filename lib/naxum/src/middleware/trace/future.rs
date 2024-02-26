use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use pin_project_lite::pin_project;
use tracing::Span;

use crate::response::Response;

use super::on_response::OnResponse;

pin_project! {
    pub struct ResponseFuture<F, OnResponse> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
        pub(crate) on_response: Option<OnResponse>,
        pub(crate) start: Instant,
    }
}

impl<Fut, E, OnResponseT> Future for ResponseFuture<Fut, OnResponseT>
where
    Fut: Future<Output = Result<Response, E>>,
    E: fmt::Display + 'static,
    OnResponseT: OnResponse,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = futures::ready!(this.inner.poll(cx));
        let latency = this.start.elapsed();

        match result {
            Ok(res) => {
                this.on_response
                    .take()
                    .unwrap()
                    .on_response(&res, latency, this.span);

                // TODO(fnichol): we need to propagate the span, which seems to infer that we need
                // some kind of response struct, even if it's used to extend to the lifetime of the
                // processing

                Poll::Ready(Ok(res))
            }
            Err(err) => {
                // TODO(fnichol): is logging appropriate here?
                Poll::Ready(Err(err))
            }
        }
    }
}
