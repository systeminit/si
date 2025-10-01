use std::sync::Arc;

use async_nats::StatusCode;
use futures::future::BoxFuture;
use tracing::error;

use super::Info;
use crate::Head;

pub trait OnFailure: Clone {
    fn call(
        &mut self,
        head: Arc<Head>,
        info: Arc<Info>,
        status: Option<StatusCode>,
    ) -> BoxFuture<'static, ()>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOnFailure {}

impl DefaultOnFailure {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OnFailure for DefaultOnFailure {
    fn call(
        &mut self,
        head: Arc<Head>,
        info: Arc<Info>,
        _status: Option<StatusCode>,
    ) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            error!(
                subject = head.subject.as_str(),
                stream_sequence = info.stream_sequence,
                "message on failure",
            );
        })
    }
}
