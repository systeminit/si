pub mod action;
pub mod async_route;
pub mod attribute;
pub mod change_set;
pub mod component;
pub mod diagram;
pub mod force_change_set_response;
pub mod graphviz;
pub mod module;
pub mod node_debug;
pub mod public;
pub mod qualification;
pub mod secret;
pub mod session;
pub mod v2;
pub mod variant;
pub mod whoami;
pub mod ws;

/// A module containing dev routes for local development only.
#[cfg(debug_assertions)]
pub mod dev;

macro_rules! impl_default_error_into_response {
    (
        $(#[$($attrss:tt)*])*
        $error_type:ident
    ) => {
        impl ::axum::response::IntoResponse for $error_type {
            fn into_response(self) -> ::axum::response::Response {
                let (status, error_message) = (
                    ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    self.to_string(),
                );

                let body = ::axum::Json(
                    ::serde_json::json!({
                        "error": {
                            "message": error_message,
                            "code": 42,
                            "statusCode": status.as_u16()
                        }
                    }),
                );

                if status.is_client_error() {
                        ::telemetry::prelude::tracing::warn!(si.error.message = error_message);
                } else if status.is_server_error() {
                        ::telemetry::prelude::tracing::warn!(si.error.message = error_message);
                }

                (status, body).into_response()
            }
        }
    };
}

pub(crate) use impl_default_error_into_response;
