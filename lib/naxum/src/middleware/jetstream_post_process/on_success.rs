use std::sync::Arc;

use async_nats::StatusCode;
use futures::future::BoxFuture;
use tracing::trace;

use super::Info;
use crate::Head;

pub trait OnSuccess: Clone {
    fn call(
        &mut self,
        head: Arc<Head>,
        info: Arc<Info>,
        status: StatusCode,
    ) -> BoxFuture<'static, ()>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOnSuccess {}

impl DefaultOnSuccess {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OnSuccess for DefaultOnSuccess {
    fn call(
        &mut self,
        _head: Arc<Head>,
        _info: Arc<Info>,
        _status: StatusCode,
    ) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            trace!("message on success");
        })
    }
}
