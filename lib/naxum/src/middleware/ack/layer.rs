use std::time::Duration;

use tower::Layer;

use super::{on_failure::DefaultOnFailure, on_success::DefaultOnSuccess, service::Ack};

// Default `ack_wait` period when unset is 30 seconds (a NATS server default)
const DEFAULT_PROGRESS_PERIOD: Duration = Duration::from_secs(30 - 1);

pub struct AckLayer<OnSuccess = DefaultOnSuccess, OnFailure = DefaultOnFailure> {
    pub(crate) on_success: OnSuccess,
    pub(crate) on_failure: OnFailure,
    pub(crate) progress_period: Duration,
}

impl Default for AckLayer {
    fn default() -> Self {
        Self {
            on_success: Default::default(),
            on_failure: Default::default(),
            progress_period: DEFAULT_PROGRESS_PERIOD,
        }
    }
}

impl AckLayer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<OnSuccess, OnFailure> AckLayer<OnSuccess, OnFailure> {
    pub fn on_success<NewOnSuccess>(
        self,
        new_on_success: NewOnSuccess,
    ) -> AckLayer<NewOnSuccess, OnFailure> {
        let Self {
            on_success: _,
            on_failure,
            progress_period,
        } = self;
        AckLayer {
            on_success: new_on_success,
            on_failure,
            progress_period,
        }
    }

    pub fn on_failure<NewOnFailure>(
        self,
        new_on_failure: NewOnFailure,
    ) -> AckLayer<OnSuccess, NewOnFailure> {
        let Self {
            on_success,
            on_failure: _,
            progress_period,
        } = self;
        AckLayer {
            on_success,
            on_failure: new_on_failure,
            progress_period,
        }
    }

    pub fn progress_period(self, new_progress_period: Duration) -> Self {
        let Self {
            on_success,
            on_failure,
            progress_period: _,
        } = self;
        AckLayer {
            on_success,
            on_failure,
            progress_period: new_progress_period,
        }
    }
}

impl<S> Layer<S> for AckLayer {
    type Service = Ack<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Ack {
            inner,
            on_success: self.on_success.clone(),
            on_failure: self.on_failure.clone(),
            progress_period: self.progress_period,
        }
    }
}
