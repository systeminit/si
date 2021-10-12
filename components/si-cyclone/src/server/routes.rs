use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{handler::get, routing::BoxRoute, AddExtensionLayer, Router};
use tracing::debug;

use super::{handlers, Config};

pub struct State {
    lang_server_path: PathBuf,
}

impl State {
    /// Gets a reference to the state's lang server path.
    pub fn lang_server_path(&self) -> &Path {
        &self.lang_server_path
    }
}

impl From<&Config> for State {
    fn from(value: &Config) -> Self {
        Self {
            lang_server_path: value.lang_server_path().to_path_buf(),
        }
    }
}

pub fn routes(config: &Config) -> Router<BoxRoute> {
    let shared_state = Arc::new(State::from(config));

    Router::new()
        .route(
            "/liveness",
            get(handlers::liveness).head(handlers::liveness),
        )
        .route(
            "/readiness",
            get(handlers::readiness).head(handlers::readiness),
        )
        .nest("/execute", execute_routes(&config))
        .layer(AddExtensionLayer::new(shared_state))
        .boxed()
}

fn execute_routes(config: &Config) -> Router<BoxRoute> {
    let mut router = Router::new().boxed();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router
            .route("/ping", get(handlers::ws_execute_ping))
            .boxed();
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router
            .route("/resolver", get(handlers::ws_execute_resolver))
            .boxed();
    }

    router
}
