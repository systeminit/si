use std::{
    fmt,
    task::{Context, Poll},
    time::Instant,
};

use tower::Service;

use crate::{response::Response, MessageHead};

use super::{
    future::ResponseFuture, make_span::MakeSpan, on_request::OnRequest, on_response::OnResponse,
    DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};

#[derive(Clone, Copy, Debug)]
pub struct Trace<
    S,
    MakeSpan = DefaultMakeSpan,
    OnRequest = DefaultOnRequest,
    OnResponse = DefaultOnResponse,
> {
    pub(crate) inner: S,
    pub(crate) make_span: MakeSpan,
    pub(crate) on_request: OnRequest,
    pub(crate) on_response: OnResponse,
}

impl<S> Trace<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            make_span: DefaultMakeSpan::new(),
            on_request: DefaultOnRequest::default(),
            on_response: DefaultOnResponse::default(),
        }
    }

    pub fn layer() -> TraceLayer {
        TraceLayer::new()
    }
}

impl<S, MakeSpan, OnRequest, OnResponse> Trace<S, MakeSpan, OnRequest, OnResponse> {
    pub fn on_request<NewOnRequest>(
        self,
        new_on_request: NewOnRequest,
    ) -> Trace<S, MakeSpan, NewOnRequest, OnResponse> {
        let Self {
            inner,
            make_span,
            on_request: _,
            on_response,
        } = self;
        Trace {
            inner,
            make_span,
            on_request: new_on_request,
            on_response,
        }
    }

    pub fn on_response<NewOnResponse>(
        self,
        new_on_response: NewOnResponse,
    ) -> Trace<S, MakeSpan, OnRequest, NewOnResponse> {
        let Self {
            inner,
            make_span,
            on_request,
            on_response: _,
        } = self;
        Trace {
            inner,
            make_span,
            on_request,
            on_response: new_on_response,
        }
    }

    pub fn make_span_with<NewMakeSpan>(
        self,
        new_make_span: NewMakeSpan,
    ) -> Trace<S, NewMakeSpan, OnRequest, OnResponse> {
        let Self {
            inner,
            make_span: _,
            on_request,
            on_response,
        } = self;
        Trace {
            inner,
            make_span: new_make_span,
            on_request,
            on_response,
        }
    }
}

impl<S, R, NewMakeSpanT, OnRequestT, OnResponseT> Service<R>
    for Trace<S, NewMakeSpanT, OnRequestT, OnResponseT>
where
    S: Service<R, Response = Response>,
    S::Error: fmt::Display + 'static,
    NewMakeSpanT: MakeSpan<R>,
    OnRequestT: OnRequest<R>,
    OnResponseT: OnResponse + Clone,
    R: MessageHead,
{
    type Response = Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, OnResponseT>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let start = Instant::now();

        let span = self.make_span.make_span(&req);

        let future = {
            let _guard = span.enter();
            self.on_request.on_request(&req, &span);
            self.inner.call(req)
        };

        ResponseFuture {
            inner: future,
            span,
            on_response: Some(self.on_response.clone()),
            start,
        }
    }
}
