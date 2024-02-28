use tracing::{Level, Span};

use crate::MessageHead;

use super::DEFAULT_MESSAGE_LEVEL;

pub trait MakeSpan<R> {
    fn make_span(&mut self, req: &R) -> Span;
}

impl<R> MakeSpan<R> for Span {
    fn make_span(&mut self, _req: &R) -> Span {
        self.clone()
    }
}

impl<F, R> MakeSpan<R> for F
where
    F: FnMut(&R) -> Span,
{
    fn make_span(&mut self, req: &R) -> Span {
        self(req)
    }
}

#[derive(Clone, Debug)]
pub struct DefaultMakeSpan {
    level: Level,
    include_headers: bool,
}

impl DefaultMakeSpan {
    pub fn new() -> Self {
        Self {
            level: DEFAULT_MESSAGE_LEVEL,
            include_headers: false,
        }
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn include_headers(mut self, include_headers: bool) -> Self {
        self.include_headers = include_headers;
        self
    }
}

impl Default for DefaultMakeSpan {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> MakeSpan<R> for DefaultMakeSpan
where
    R: MessageHead,
{
    fn make_span(&mut self, req: &R) -> Span {
        let reply = req.reply().map(|r| r.as_str()).unwrap_or_default();
        let status = req.status().map(|s| s.as_u16()).unwrap_or_default();

        // This ugly macro is needed, unfortunately, because `tracing::span!`
        // required the level argument to be static. Meaning we can't just pass
        // `self.level`.
        macro_rules! make_span {
            ($level:expr) => {
                if self.include_headers {
                    tracing::span!(
                        $level,
                        "receive message",
                        subject = req.subject().as_str(),
                        reply = reply,
                        status = status,
                        length = req.length(),
                        headers = ?req.headers(),
                    )
                } else {
                    tracing::span!(
                        $level,
                        "receive message",
                        subject = req.subject().as_str(),
                        reply = reply,
                        status = status,
                        length = req.length(),
                    )
                }
            }
        }

        match self.level {
            Level::ERROR => make_span!(Level::ERROR),
            Level::WARN => make_span!(Level::WARN),
            Level::INFO => make_span!(Level::INFO),
            Level::DEBUG => make_span!(Level::DEBUG),
            Level::TRACE => make_span!(Level::TRACE),
        }
    }
}
