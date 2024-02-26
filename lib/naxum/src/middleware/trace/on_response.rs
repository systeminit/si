use std::time::Duration;

use tracing::{Level, Span};

use crate::{
    middleware::{trace::Latency, LatencyUnit},
    response::Response,
};

use super::DEFAULT_MESSAGE_LEVEL;

pub trait OnResponse {
    fn on_response(self, response: &Response, latency: Duration, span: &Span);
}

impl OnResponse for () {
    #[inline]
    fn on_response(self, _response: &Response, _latency: Duration, _span: &Span) {}
}

impl<F> OnResponse for F
where
    F: FnOnce(&Response, Duration, &Span),
{
    fn on_response(self, response: &Response, latency: Duration, span: &Span) {
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

impl OnResponse for DefaultOnResponse {
    fn on_response(self, _response: &Response, latency: Duration, _span: &Span) {
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
