pub use config::{
    Config, ConfigBuilder, ConfigError, ConfigFile, IncomingStream, JwtSecretKey, StandardConfig,
    StandardConfigFile,
};
pub use dal::{JobQueueProcessor, MigrationMode, NatsProcessor, SyncProcessor};
pub use routes::{routes, AppError, AppResult};
pub use server::{build_service, Server};
pub use uds::{UdsIncomingStream, UdsIncomingStreamError};

mod config;
pub(crate) mod extract;
mod handlers;
pub(crate) mod job_processor;
mod routes;
mod server;
pub mod service;
mod uds;

macro_rules! impl_default_error_into_response {
    (
        $(#[$($attrss:tt)*])*
        $error_type:ident
    ) => {
        impl axum::response::IntoResponse for $error_type {
            fn into_response(self) -> Response {
                let (status, error_message) = (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

                let body = Json(
                    serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
                );

                (status, body).into_response()
            }
        }
    };
}

pub(crate) use impl_default_error_into_response;
