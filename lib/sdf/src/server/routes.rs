use std::sync::Arc;

use axum::{routing::BoxRoute, AddExtensionLayer, Router};
use tokio::sync::mpsc;

use super::{server::ShutdownSource, Config};

#[allow(dead_code)]
struct State {
    // TODO(fnichol): we're likely going to use this, but we can't allow it to be dropped because
    // that will trigger the read side and... shutdown. Cool, no?
    tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl State {
    fn new(tmp_shutdown_tx: mpsc::Sender<ShutdownSource>) -> Self {
        Self { tmp_shutdown_tx }
    }
}

#[must_use]
pub fn routes(_config: &Config, shutdown_tx: mpsc::Sender<ShutdownSource>) -> Router<BoxRoute> {
    let shared_state = Arc::new(State::new(shutdown_tx));

    let router = Router::new().boxed();

    router.layer(AddExtensionLayer::new(shared_state)).boxed()
}
