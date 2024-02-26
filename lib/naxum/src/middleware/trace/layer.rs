use tower::Layer;

use super::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, Trace};

pub struct TraceLayer<
    MakeSpan = DefaultMakeSpan,
    OnRequest = DefaultOnRequest,
    OnResponse = DefaultOnResponse,
> {
    pub(crate) make_span: MakeSpan,
    pub(crate) on_request: OnRequest,
    pub(crate) on_response: OnResponse,
}

impl Default for TraceLayer {
    fn default() -> Self {
        Self {
            make_span: DefaultMakeSpan::new(),
            on_request: DefaultOnRequest::default(),
            on_response: DefaultOnResponse::default(),
        }
    }
}
impl TraceLayer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<MakeSpan, OnRequest, OnResponse> TraceLayer<MakeSpan, OnRequest, OnResponse> {
    pub fn on_request<NewOnRequest>(
        self,
        new_on_request: NewOnRequest,
    ) -> TraceLayer<MakeSpan, NewOnRequest, OnResponse> {
        let Self {
            make_span,
            on_request: _,
            on_response,
        } = self;
        TraceLayer {
            make_span,
            on_request: new_on_request,
            on_response,
        }
    }

    pub fn on_response<NewOnResponse>(
        self,
        new_on_response: NewOnResponse,
    ) -> TraceLayer<MakeSpan, OnRequest, NewOnResponse> {
        let Self {
            make_span,
            on_request,
            on_response: _,
        } = self;
        TraceLayer {
            make_span,
            on_request,
            on_response: new_on_response,
        }
    }

    pub fn make_span_with<NewMakeSpan>(
        self,
        new_make_span: NewMakeSpan,
    ) -> TraceLayer<NewMakeSpan, OnRequest, OnResponse> {
        let Self {
            make_span: _,
            on_request,
            on_response,
        } = self;
        TraceLayer {
            make_span: new_make_span,
            on_request,
            on_response,
        }
    }
}

impl<S, MakeSpan, OnRequest, OnResponse> Layer<S> for TraceLayer<MakeSpan, OnRequest, OnResponse>
where
    MakeSpan: Clone,
    OnRequest: Clone,
    OnResponse: Clone,
{
    type Service = Trace<S, MakeSpan, OnRequest, OnResponse>;

    fn layer(&self, inner: S) -> Self::Service {
        Trace {
            inner,
            make_span: self.make_span.clone(),
            on_request: self.on_request.clone(),
            on_response: self.on_response.clone(),
        }
    }
}
