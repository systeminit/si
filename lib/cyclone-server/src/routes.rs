use std::sync::Arc;

use axum::{routing::get, Extension, Router};
use telemetry::prelude::*;
use tokio::sync::mpsc;
use tower_http::compression::CompressionLayer;

use crate::{
    extract::RequestLimiter,
    handlers,
    state::{AppState, WatchKeepalive},
    watch, Config, ShutdownSource,
};

pub fn routes(
    config: &Config,
    state: AppState,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
) -> Router {
    let mut router: Router<AppState> = Router::new()
        .route(
            "/liveness",
            get(handlers::liveness).head(handlers::liveness),
        )
        .route(
            "/readiness",
            get(handlers::readiness).head(handlers::readiness),
        )
        .nest("/execute", execute_routes(config, shutdown_tx.clone()))
        .layer(CompressionLayer::new());

    if let Some(watch_timeout) = config.watch() {
        debug!("enabling watch endpoint");
        let (keepalive_tx, keepalive_rx) = mpsc::channel::<()>(4);

        tokio::spawn(watch::watch_timeout_task(
            watch_timeout,
            shutdown_tx,
            keepalive_rx,
        ));

        let watch_keepalive = WatchKeepalive::new(keepalive_tx, watch_timeout);

        router = router.merge(
            Router::new()
                .route("/watch", get(handlers::ws_watch))
                .layer(Extension(Arc::new(watch_keepalive))),
        );
    }

    router.with_state(state)
}

fn execute_routes(config: &Config, shutdown_tx: mpsc::Sender<ShutdownSource>) -> Router<AppState> {
    let mut router = Router::new();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router.merge(Router::new().route("/ping", get(handlers::ws_execute_ping)));
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router.merge(Router::new().route("/resolver", get(handlers::ws_execute_resolver)));
    }
    if config.enable_validation() {
        debug!("enabling validation endpoint");
        router =
            router.merge(Router::new().route("/validation", get(handlers::ws_execute_validation)));
    }
    if config.enable_action_run() {
        debug!("enabling command run endpoint");
        router =
            router.merge(Router::new().route("/command", get(handlers::ws_execute_action_run)));
    }
    if config.enable_reconciliation() {
        debug!("enabling reconciliation endpoint");
        router = router.merge(
            Router::new().route("/reconciliation", get(handlers::ws_execute_reconciliation)),
        );
    }
    if config.enable_schema_variant_definition() {
        debug!("enabling schema variant definition endpoint");
        router = router.merge(Router::new().route(
            "/schema_variant_definition",
            get(handlers::ws_execute_schema_variant_definition),
        ));
    }

    let limit_requests = Arc::new(config.limit_requests().map(|i| i.into()));

    router.layer(Extension(RequestLimiter::new(limit_requests, shutdown_tx)))
}
