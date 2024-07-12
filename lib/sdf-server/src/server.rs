pub use config::{
    detect_and_configure_development, Config, ConfigBuilder, ConfigError, ConfigFile,
    IncomingStream, StandardConfig, StandardConfigFile,
};
pub use dal::{
    context::SystemActor, JobQueueProcessor, MigrationMode, NatsProcessor, ServicesContext,
};
pub use nats_multiplexer::CRDT_MULTIPLEXER_SUBJECT;
pub use nats_multiplexer::WS_MULTIPLEXER_SUBJECT;
pub use routes::{routes, AppError};
pub use server::{build_service, build_service_for_tests, Server};
pub use si_data_pg::PgPool;
pub use si_layer_cache::LayerDb;
pub use uds::{UdsIncomingStream, UdsIncomingStreamError};

mod config;
pub(crate) mod extract;
pub(crate) mod job_processor;
mod nats_multiplexer;
mod routes;
mod server;
pub mod service;
pub mod state;
pub mod tracking;
mod uds;

macro_rules! impl_default_error_into_response {
    (
        $(#[$($attrss:tt)*])*
        $error_type:ident
    ) => {
        impl axum::response::IntoResponse for $error_type {
            fn into_response(self) -> Response {
                let (status, error_message) = match self {
                   $error_type::Transactions(TransactionsError::ConflictsOccurred(_)) => (axum::http::StatusCode::CONFLICT, self.to_string()),
                  _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
                };

                let body = Json(
                    serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
                );

                (status, body).into_response()
            }
        }
    };
}

pub(crate) use impl_default_error_into_response;
