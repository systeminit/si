use tower::Layer;

use super::{
    DefaultOnFailure,
    DefaultOnSuccess,
    PostProcess,
};

pub struct PostProcessLayer<OnSuccess = DefaultOnSuccess, OnFailure = DefaultOnFailure> {
    pub(crate) on_success: OnSuccess,
    pub(crate) on_failure: OnFailure,
}

impl Default for PostProcessLayer {
    fn default() -> Self {
        Self {
            on_success: Default::default(),
            on_failure: Default::default(),
        }
    }
}

impl PostProcessLayer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<OnSuccess, OnFailure> PostProcessLayer<OnSuccess, OnFailure> {
    pub fn on_success<NewOnSuccess>(
        self,
        new_on_success: NewOnSuccess,
    ) -> PostProcessLayer<NewOnSuccess, OnFailure> {
        let Self {
            on_success: _,
            on_failure,
        } = self;
        PostProcessLayer {
            on_success: new_on_success,
            on_failure,
        }
    }

    pub fn on_failure<NewOnFailure>(
        self,
        new_on_failure: NewOnFailure,
    ) -> PostProcessLayer<OnSuccess, NewOnFailure> {
        let Self {
            on_success,
            on_failure: _,
        } = self;
        PostProcessLayer {
            on_success,
            on_failure: new_on_failure,
        }
    }
}

impl<S, OnSuccess, OnFailure> Layer<S> for PostProcessLayer<OnSuccess, OnFailure>
where
    OnSuccess: Clone,
    OnFailure: Clone,
{
    type Service = PostProcess<S, OnSuccess, OnFailure>;

    fn layer(&self, inner: S) -> Self::Service {
        PostProcess {
            inner,
            on_success: self.on_success.clone(),
            on_failure: self.on_failure.clone(),
        }
    }
}
