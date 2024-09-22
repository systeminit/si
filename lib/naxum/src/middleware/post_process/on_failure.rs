use std::sync::Arc;

use futures::future::BoxFuture;
use tracing::trace;

use crate::Head;

use super::Info;

pub trait OnFailure {
    fn call(&mut self, head: Arc<Head>, info: Arc<Info>) -> BoxFuture<'static, ()>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOnFailure {}

impl DefaultOnFailure {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OnFailure for DefaultOnFailure {
    fn call(&mut self, _head: Arc<Head>, _info: Arc<Info>) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            trace!("message on failure");
        })
    }
}
