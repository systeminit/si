use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use axum::{handler::get, routing::BoxRoute, AddExtensionLayer, Router};
use telemetry::{prelude::*, TelemetryLevel};
use tokio::sync::mpsc;

use super::{
    decryption_key::DecryptionKey, extract::RequestLimiter, handlers, server::ShutdownSource,
    Config,
};
use crate::server::watch;

#[derive(Debug)]
pub struct LangServerPath(PathBuf);

impl LangServerPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

pub struct WatchKeepalive {
    tx: mpsc::Sender<()>,
    timeout: Duration,
}

impl WatchKeepalive {
    pub fn clone_tx(&self) -> mpsc::Sender<()> {
        self.tx.clone()
    }

    /// Gets a reference to the watch keepalive tx's timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[must_use]
pub fn routes(
    config: &Config,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
    telemetry_level: Box<dyn TelemetryLevel>,
    decryption_key: DecryptionKey,
) -> Router<BoxRoute> {
    let lang_server_path = LangServerPath::new(config.lang_server_path());

    let mut router = Router::new()
        .route(
            "/liveness",
            get(handlers::liveness).head(handlers::liveness),
        )
        .route(
            "/readiness",
            get(handlers::readiness).head(handlers::readiness),
        )
        .nest("/execute", execute_routes(config, shutdown_tx.clone()))
        .boxed();

    if let Some(watch_timeout) = config.watch() {
        debug!("enabling watch endpoint");
        let (keepalive_tx, keepalive_rx) = mpsc::channel::<()>(4);

        tokio::spawn(watch::watch_timeout_task(
            watch_timeout,
            shutdown_tx,
            keepalive_rx,
        ));

        let watch_keepalive = WatchKeepalive {
            tx: keepalive_tx,
            timeout: watch_timeout,
        };

        router = router
            .or(Router::new()
                .route("/watch", get(handlers::ws_watch))
                .layer(AddExtensionLayer::new(Arc::new(watch_keepalive))))
            .boxed();
    }

    router = router
        .layer(AddExtensionLayer::new(Arc::new(lang_server_path)))
        .layer(AddExtensionLayer::new(Arc::new(decryption_key)))
        .layer(AddExtensionLayer::new(Arc::new(telemetry_level)))
        .boxed();
    router
}

fn execute_routes(config: &Config, shutdown_tx: mpsc::Sender<ShutdownSource>) -> Router<BoxRoute> {
    let mut router = Router::new().boxed();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router
            .or(Router::new().route("/ping", get(handlers::ws_execute_ping)))
            .boxed();
    }
    if config.enable_qualification() {
        debug!("enabling qualification endpoint");
        router = router
            .or(Router::new().route("/qualification", get(handlers::ws_execute_qualification)))
            .boxed();
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router
            .or(Router::new().route("/resolver", get(handlers::ws_execute_resolver)))
            .boxed();
    }
    if config.enable_sync() {
        debug!("enabling sync endpoint");
        router = router
            .or(Router::new().route("/sync", get(handlers::ws_execute_sync)))
            .boxed();
    }
    if config.enable_code_generation() {
        debug!("enabling code generation endpoint");
        router = router
            .or(Router::new().route(
                "/code_generation",
                get(handlers::ws_execute_code_generation),
            ))
            .boxed();
    }

    let limit_requests = Arc::new(config.limit_requests().map(|i| i.into()));

    router
        .layer(AddExtensionLayer::new(RequestLimiter::new(
            limit_requests,
            shutdown_tx,
        )))
        .boxed()
}
