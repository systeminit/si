use std::{net::TcpStream, sync::Arc};

use crate::server::config::JwtSecretKey;
use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use dal::context::{FaktoryProducer, ServicesContext};
use hyper::StatusCode;
use si_data::{nats, pg, SensitiveString};
use telemetry::TelemetryClient;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, Mutex};

use super::{
    handlers,
    server::{ServerError, ShutdownSource},
};

pub struct State {
    // TODO(fnichol): we're likely going to use this, but we can't allow it to be dropped because
    // that will trigger the read side and... shutdown. Cool, no?
    #[allow(dead_code)]
    tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl State {
    pub fn new(tmp_shutdown_tx: mpsc::Sender<ShutdownSource>) -> Self {
        Self { tmp_shutdown_tx }
    }
}

#[derive(Clone, Debug)]
pub struct ShutdownBroadcast(broadcast::Sender<()>);

impl ShutdownBroadcast {
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.0.subscribe()
    }
}

#[derive(Clone, Debug)]
pub struct SignupSecret(Arc<SensitiveString>);

impl SignupSecret {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn routes(
    telemetry: impl TelemetryClient,
    pg_pool: pg::PgPool,
    nats_conn: nats::Client,
    faktory_conn: FaktoryProducer,
    veritech: veritech::Client,
    encryption_key: veritech::EncryptionKey,
    jwt_secret_key: JwtSecretKey,
    signup_secret: SensitiveString,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
) -> Router {
    let shared_state = Arc::new(State::new(shutdown_tx));
    let encryption_key = Arc::new(encryption_key);
    let signup_secret = SignupSecret(Arc::new(signup_secret));
    let services_context = ServicesContext::new(
        pg_pool.clone(),
        nats_conn.clone(),
        faktory_conn,
        veritech.clone(),
        encryption_key.clone(),
    );

    let mut router: Router = Router::new();
    router = router
        .route("/api/demo", get(handlers::demo))
        .nest(
            "/api/application",
            crate::server::service::application::routes(),
        )
        .nest(
            "/api/change_set",
            crate::server::service::change_set::routes(),
        )
        .nest(
            "/api/component",
            crate::server::service::component::routes(),
        )
        .nest("/api/provider", crate::server::service::provider::routes())
        .nest(
            "/api/qualification",
            crate::server::service::qualification::routes(),
        )
        .nest("/api/schema", crate::server::service::schema::routes())
        .nest(
            "/api/schematic",
            crate::server::service::schematic::routes(),
        )
        .nest("/api/secret", crate::server::service::secret::routes())
        .nest("/api/session", crate::server::service::session::routes())
        .nest("/api/signup", crate::server::service::signup::routes())
        .nest("/api/system", crate::server::service::system::routes())
        .nest("/api/ws", crate::server::service::ws::routes());
    router = test_routes(router);
    router = router
        .layer(Extension(shared_state))
        .layer(Extension(services_context))
        .layer(Extension(telemetry))
        .layer(Extension(pg_pool))
        .layer(Extension(nats_conn))
        .layer(Extension(veritech))
        .layer(Extension(encryption_key))
        .layer(Extension(jwt_secret_key))
        .layer(Extension(signup_secret))
        .layer(Extension(ShutdownBroadcast(shutdown_broadcast_tx)));
    router
}

#[cfg(debug_assertions)]
pub fn test_routes(mut router: Router) -> Router {
    router = router.nest("/api/test", crate::server::service::test::routes());
    router
}

#[cfg(not(debug_assertions))]
pub fn test_routes(mut router: Router) -> Router {
    router
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] nats::Error),
    #[error(transparent)]
    Pg(#[from] pg::Error),
    #[error(transparent)]
    Server(#[from] ServerError),
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16(),
            },
        }));

        (status, body).into_response()
    }
}
