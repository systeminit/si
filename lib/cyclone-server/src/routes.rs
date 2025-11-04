use std::sync::Arc;

use axum::{
    Extension,
    Router,
    routing::get,
};
use telemetry::prelude::*;
use telemetry_http::{
    HttpMakeSpan,
    HttpOnResponse,
};
use tokio::sync::mpsc;
use tower_http::{
    compression::CompressionLayer,
    trace::TraceLayer,
};

use crate::{
    Config,
    ShutdownSource,
    extract::RequestLimiter,
    handlers,
    state::{
        AppState,
        WatchKeepalive,
    },
    tower::WebSocketTraceLayer,
    watch,
};

pub fn routes(
    config: &Config,
    state: AppState,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
) -> Router {
    let http_trace_layer = TraceLayer::new_for_http()
        .make_span_with(HttpMakeSpan::builder().level(Level::INFO).build())
        .on_response(HttpOnResponse::new().level(Level::DEBUG));
    let web_socket_trace_layer = WebSocketTraceLayer::new();

    let mut router: Router<AppState> = Router::new()
        .route(
            "/liveness",
            get(handlers::liveness)
                .head(handlers::liveness)
                .layer(http_trace_layer.clone()),
        )
        .route(
            "/readiness",
            get(handlers::readiness)
                .head(handlers::readiness)
                .layer(http_trace_layer.clone()),
        )
        .nest(
            "/execute",
            execute_routes(config, shutdown_tx.clone())
                .layer(http_trace_layer.clone())
                .layer(web_socket_trace_layer),
        )
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
                .nest(
                    "/watch",
                    Router::new()
                        .route("/", get(handlers::ws_watch))
                        .layer(http_trace_layer),
                )
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
    if config.enable_schema_variant_definition() {
        debug!("enabling schema variant definition endpoint");
        router = router.merge(Router::new().route(
            "/schema_variant_definition",
            get(handlers::ws_execute_schema_variant_definition),
        ));
    }
    if config.enable_management() {
        debug!("enabling management function endpoint");
        router =
            router.merge(Router::new().route("/management", get(handlers::ws_execute_management)));
    }
    if config.enable_debug() {
        debug!("enabling debug function endpoint");
        router = router.merge(Router::new().route("/debug", get(handlers::ws_execute_debug)));
    }

    let limit_requests = Arc::new(config.limit_requests().map(|i| i.into()));

    router.layer(Extension(RequestLimiter::new(limit_requests, shutdown_tx)))
}
