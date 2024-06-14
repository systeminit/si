use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use pin_project_lite::pin_project;
use tracing::Span;

use crate::{body, response};

use super::{on_response::OnResponse, ResponseBody};

pin_project! {
    pub struct ResponseFuture<F, OnResponse> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
        pub(crate) on_response: Option<OnResponse>,
        pub(crate) start: Instant,
    }
}

impl<Fut, ResBody, E, OnResponseT> Future for ResponseFuture<Fut, OnResponseT>
where
    Fut: Future<Output = Result<response::inner::Response<ResBody>, E>>,
    ResBody: body::inner::Body,
    E: fmt::Display + 'static,
    OnResponseT: OnResponse<ResBody>,
{
    type Output = Result<response::inner::Response<ResponseBody<ResBody>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = futures::ready!(this.inner.poll(cx));
        let latency = this.start.elapsed();

        match result {
            Ok(res) => {
                let start = *this.start;

                this.on_response
                    .take()
                    .unwrap()
                    .on_response(&res, latency, this.span);

                let span = this.span.clone();
                let res = res.map(|body| ResponseBody {
                    inner: body,
                    _start: start,
                    _span: span,
                });

                Poll::Ready(Ok(res))
            }
            Err(err) => {
                // TODO(fnichol): is logging appropriate here?
                Poll::Ready(Err(err))
            }
        }
    }
}
