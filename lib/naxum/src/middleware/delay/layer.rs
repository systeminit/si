use std::time::Duration;

use tower::Layer;

use super::service::Delay;

pub struct DelayLayer {
    pub(crate) wait: Duration,
}

impl DelayLayer {
    pub fn new(wait: Duration) -> Self {
        Self { wait }
    }
}

impl<S> Layer<S> for DelayLayer {
    type Service = Delay<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Delay {
            inner,
            wait: self.wait,
        }
    }
}
