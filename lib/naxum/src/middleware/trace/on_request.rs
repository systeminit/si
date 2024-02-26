use tracing::{Level, Span};

use super::DEFAULT_MESSAGE_LEVEL;

pub trait OnRequest<R> {
    fn on_request(&mut self, req: &R, span: &Span);
}

impl<R> OnRequest<R> for () {
    fn on_request(&mut self, _req: &R, _span: &Span) {}
}

impl<R, F> OnRequest<R> for F
where
    F: FnMut(&R, &Span),
{
    fn on_request(&mut self, req: &R, span: &Span) {
        self(req, span)
    }
}

#[derive(Clone, Debug)]
pub struct DefaultOnRequest {
    level: Level,
}

impl Default for DefaultOnRequest {
    fn default() -> Self {
        Self {
            level: DEFAULT_MESSAGE_LEVEL,
        }
    }
}

impl DefaultOnRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl<R> OnRequest<R> for DefaultOnRequest {
    fn on_request(&mut self, _req: &R, _span: &Span) {
        event_dynamic_lvl!(self.level, "started processing message");
    }
}
