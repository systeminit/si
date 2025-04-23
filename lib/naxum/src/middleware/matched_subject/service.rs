use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use tower::Service;

use super::{
    DefaultForSubject,
    ForSubject,
    MatchedSubjectLayer,
};
use crate::{
    Message,
    MessageHead,
};

#[derive(Clone, Copy, Debug)]
pub struct MatchedSubject<S, ForSubject = DefaultForSubject> {
    pub(crate) inner: S,
    pub(crate) for_subject: ForSubject,
}

impl<S> MatchedSubject<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            for_subject: DefaultForSubject::default(),
        }
    }

    pub fn layer() -> MatchedSubjectLayer {
        MatchedSubjectLayer::new()
    }
}

impl<S, ForSubjectT, R> Service<Message<R>> for MatchedSubject<S, ForSubjectT>
where
    S: Service<Message<R>> + Clone + Send + 'static,
    S::Future: Send,
    ForSubjectT: ForSubject<R>,
    R: MessageHead + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Message<R>) -> Self::Future {
        self.for_subject.call(&mut req);

        let clone = self.inner.clone();
        // Take the service that was ready
        //
        // See documentation for [`Service`] trait
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move { inner.call(req).await })
    }
}
