use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use axum::{routing::get, Extension, Router};
use telemetry::{prelude::*, TelemetryLevel};
use tokio::sync::mpsc;

use crate::{extract::RequestLimiter, handlers, watch, Config, DecryptionKey, ShutdownSource};

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
) -> Router {
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
        .nest("/execute", execute_routes(config, shutdown_tx.clone()));

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

        router = router.merge(
            Router::new()
                .route("/watch", get(handlers::ws_watch))
                .layer(Extension(Arc::new(watch_keepalive))),
        );
    }

    router = router
        .layer(Extension(Arc::new(lang_server_path)))
        .layer(Extension(Arc::new(decryption_key)))
        .layer(Extension(Arc::new(telemetry_level)));
    router
}

fn execute_routes(config: &Config, shutdown_tx: mpsc::Sender<ShutdownSource>) -> Router {
    let mut router = Router::new();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router.merge(Router::new().route("/ping", get(handlers::ws_execute_ping)));
    }
    if config.enable_qualification() {
        debug!("enabling qualification endpoint");
        router = router
            .merge(Router::new().route("/qualification", get(handlers::ws_execute_qualification)));
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router.merge(Router::new().route("/resolver", get(handlers::ws_execute_resolver)));
    }
    if config.enable_confirmation() {
        debug!("enabling confirmation endpoint");
        router = router
            .merge(Router::new().route("/confirmation", get(handlers::ws_execute_confirmation)));
    }
    if config.enable_validation() {
        debug!("enabling validation endpoint");
        router =
            router.merge(Router::new().route("/validation", get(handlers::ws_execute_validation)));
    }
    if config.enable_workflow_resolve() {
        debug!("enabling workflow resolve endpoint");
        router = router
            .merge(Router::new().route("/workflow", get(handlers::ws_execute_workflow_resolve)));
    }
    if config.enable_command_run() {
        debug!("enabling command run endpoint");
        router =
            router.merge(Router::new().route("/command", get(handlers::ws_execute_command_run)));
    }

    let limit_requests = Arc::new(config.limit_requests().map(|i| i.into()));

    router.layer(Extension(RequestLimiter::new(limit_requests, shutdown_tx)))
}
