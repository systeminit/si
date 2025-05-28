use tower::Layer;

use super::{
    DefaultOnFailure,
    DefaultOnSuccess,
    JetstreamPostProcess,
};

pub struct JetstreamPostProcessLayer<OnSuccess = DefaultOnSuccess, OnFailure = DefaultOnFailure> {
    pub(crate) on_success: OnSuccess,
    pub(crate) on_failure: OnFailure,
}

impl Default for JetstreamPostProcessLayer {
    fn default() -> Self {
        Self {
            on_success: Default::default(),
            on_failure: Default::default(),
        }
    }
}

impl JetstreamPostProcessLayer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<OnSuccess, OnFailure> JetstreamPostProcessLayer<OnSuccess, OnFailure> {
    pub fn on_success<NewOnSuccess>(
        self,
        new_on_success: NewOnSuccess,
    ) -> JetstreamPostProcessLayer<NewOnSuccess, OnFailure> {
        let Self {
            on_success: _,
            on_failure,
        } = self;
        JetstreamPostProcessLayer {
            on_success: new_on_success,
            on_failure,
        }
    }

    pub fn on_failure<NewOnFailure>(
        self,
        new_on_failure: NewOnFailure,
    ) -> JetstreamPostProcessLayer<OnSuccess, NewOnFailure> {
        let Self {
            on_success,
            on_failure: _,
        } = self;
        JetstreamPostProcessLayer {
            on_success,
            on_failure: new_on_failure,
        }
    }
}

impl<S, OnSuccess, OnFailure> Layer<S> for JetstreamPostProcessLayer<OnSuccess, OnFailure>
where
    OnSuccess: Clone,
    OnFailure: Clone,
{
    type Service = JetstreamPostProcess<S, OnSuccess, OnFailure>;

    fn layer(&self, inner: S) -> Self::Service {
        JetstreamPostProcess {
            inner,
            on_success: self.on_success.clone(),
            on_failure: self.on_failure.clone(),
        }
    }
}
