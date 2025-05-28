use std::sync::Arc;

use futures::future::BoxFuture;
use tracing::error;

use crate::Head;

pub trait OnFailure {
    fn call(&mut self, head: Arc<Head>) -> BoxFuture<'static, ()>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOnFailure {}

impl DefaultOnFailure {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OnFailure for DefaultOnFailure {
    fn call(&mut self, head: Arc<Head>) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            error!(subject = head.subject.as_str(), "message on failure",);
        })
    }
}
