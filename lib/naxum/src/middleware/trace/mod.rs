macro_rules! event_dynamic_lvl {
    ( $(target: $target:expr_2021,)? $(parent: $parent:expr_2021,)? $lvl:expr_2021, $($tt:tt)* ) => {
        match $lvl {
            tracing::Level::ERROR => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::ERROR,
                    $($tt)*
                );
            }
            tracing::Level::WARN => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::WARN,
                    $($tt)*
                );
            }
            tracing::Level::INFO => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::INFO,
                    $($tt)*
                );
            }
            tracing::Level::DEBUG => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::DEBUG,
                    $($tt)*
                );
            }
            tracing::Level::TRACE => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::TRACE,
                    $($tt)*
                );
            }
        }
    };
}

mod body;
mod future;
mod layer;
mod make_span;
mod on_request;
mod on_response;
mod service;

use std::{
    fmt,
    time::Duration,
};

use tracing::Level;

pub use self::{
    body::ResponseBody,
    layer::TraceLayer,
    make_span::{
        DefaultMakeSpan,
        MakeSpan,
    },
    on_request::{
        DefaultOnRequest,
        OnRequest,
    },
    on_response::{
        DefaultOnResponse,
        OnResponse,
    },
    service::Trace,
};
use super::LatencyUnit;

const DEFAULT_MESSAGE_LEVEL: Level = Level::DEBUG;

pub struct Latency {
    pub unit: LatencyUnit,
    pub duration: Duration,
}

impl fmt::Display for Latency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit {
            LatencyUnit::Seconds => write!(f, "{} s", self.duration.as_secs_f64()),
            LatencyUnit::Millis => write!(f, "{} ms", self.duration.as_millis()),
            LatencyUnit::Micros => write!(f, "{} μs", self.duration.as_micros()),
            LatencyUnit::Nanos => write!(f, "{} ns", self.duration.as_nanos()),
        }
    }
}
