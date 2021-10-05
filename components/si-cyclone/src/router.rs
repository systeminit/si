use crate::Config;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    handler::get,
    response::IntoResponse,
    routing::BoxRoute,
    Router,
};
use hyper::StatusCode;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, warn};

pub fn app(config: Config) -> Router<BoxRoute> {
    Router::new()
        .route("/liveness", get(liveness).head(liveness))
        .route("/readiness", get(readiness).head(readiness))
        .route("/execute", execute_routes(&config))
        // TODO(fnichol): customize http tracing further, using:
        // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .boxed()
}

#[derive(Debug)]
enum LivenessStatus {
    Ok,
}

impl LivenessStatus {
    fn as_str(&self) -> &'static str {
        match self {
            LivenessStatus::Ok => "ok\n",
        }
    }
}

impl From<LivenessStatus> for &'static str {
    fn from(value: LivenessStatus) -> Self {
        value.as_str()
    }
}

async fn liveness() -> (StatusCode, &'static str) {
    (StatusCode::OK, LivenessStatus::Ok.into())
}

#[derive(Debug)]
enum ReadinessStatus {
    Ready,
}

impl ReadinessStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ReadinessStatus::Ready => "ready\n",
        }
    }
}

impl From<ReadinessStatus> for &'static str {
    fn from(value: ReadinessStatus) -> Self {
        value.as_str()
    }
}

async fn readiness() -> Result<&'static str, StatusCode> {
    Ok(ReadinessStatus::Ready.into())
}

fn execute_routes(config: &Config) -> Router<BoxRoute> {
    let mut router = Router::new().boxed();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router.route("/ping", get(execute_ws_ping)).boxed();
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router.route("/resolver", get(execute_ws_resolver)).boxed();
    }

    router
}

async fn execute_ws_ping(ws: WebSocketUpgrade) -> impl IntoResponse {
    async fn handle_socket(mut socket: WebSocket) {
        if let Err(ref err) = socket.send(Message::Text("pong".to_string())).await {
            warn!("client disconnected; error={}", err);
        }
    }

    ws.on_upgrade(handle_socket);
}

async fn execute_ws_resolver(ws: WebSocketUpgrade) -> impl IntoResponse {
    async fn handle_socket(_socket: WebSocket) {
        todo!("HAY resolver!")
    }

    ws.on_upgrade(handle_socket);
}
