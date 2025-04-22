use std::time::Duration;

use tracing::{
    Level,
    Span,
};

use super::DEFAULT_MESSAGE_LEVEL;
use crate::{
    middleware::{
        LatencyUnit,
        trace::Latency,
    },
    response::Response,
};

pub trait OnResponse<B> {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span);
}

impl<B> OnResponse<B> for () {
    #[inline]
    fn on_response(self, _response: &Response<B>, _latency: Duration, _span: &Span) {}
}

impl<B, F> OnResponse<B> for F
where
    F: FnOnce(&Response<B>, Duration, &Span),
{
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        self(response, latency, span)
    }
}

#[derive(Clone, Debug)]
pub struct DefaultOnResponse {
    level: Level,
    latency_unit: LatencyUnit,
}

impl Default for DefaultOnResponse {
    fn default() -> Self {
        Self {
            level: DEFAULT_MESSAGE_LEVEL,
            latency_unit: LatencyUnit::Millis,
        }
    }
}

impl DefaultOnResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn latency_unit(mut self, latency_unit: LatencyUnit) -> Self {
        self.latency_unit = latency_unit;
        self
    }
}

impl<B> OnResponse<B> for DefaultOnResponse {
    fn on_response(self, _response: &Response<B>, latency: Duration, _span: &Span) {
        let latency = Latency {
            unit: self.latency_unit,
            duration: latency,
        };

        event_dynamic_lvl!(
            self.level,
            %latency,
            "finished processing mesasge"
        );
    }
}
