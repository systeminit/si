use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{handler::get, routing::BoxRoute, AddExtensionLayer, Router};
use tokio::sync::mpsc;
use tracing::debug;

use super::{handlers, tower::LimitRequestLayer, Config};

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

#[must_use]
pub fn routes(config: &Config, limit_request_shutdown_tx: mpsc::Sender<()>) -> Router<BoxRoute> {
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
        .nest(
            "/execute",
            execute_routes(config, limit_request_shutdown_tx),
        )
        .layer(AddExtensionLayer::new(shared_state))
        .boxed()
}

fn execute_routes(
    config: &Config,
    limit_request_shutdown_tx: mpsc::Sender<()>,
) -> Router<BoxRoute> {
    let mut router = Router::new().boxed();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router
            .or(Router::new().route("/ping", get(handlers::ws_execute_ping)))
            .boxed();
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router
            .or(Router::new().route("/resolver", get(handlers::ws_execute_resolver)))
            .boxed();
    }

    router
        .layer(LimitRequestLayer::new(
            config.limit_requests(),
            limit_request_shutdown_tx,
        ))
        // TODO(fnichol): we are going to need this, mark my words...
        // .handle_error(convert_tower_error_into_reponse)
        .boxed()
}

// TODO(fnichol): we are going to need this, mark my words...
//
//
// fn convert_tower_error_into_reponse(err: BoxError) -> Result<Response<Full<Bytes>>, Infallible> {
//     // TODO(fnichol): more to do here, see:
//     // https://github.com/bwalter/rust-axum-scylla/blob/main/src/routing/mod.rs
//     Ok((
//         StatusCode::INTERNAL_SERVER_ERROR,
//         Json(json!({ "error": err.to_string() })),
//     )
//         .into_response())
// }
