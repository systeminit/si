//! Core types and utilities shared by different sdf route implementations should go here

use std::{
    collections::HashMap,
    sync::Arc,
};

use tokio::sync::Mutex;

pub mod api_error;
pub mod app_state;
pub mod async_route;
pub mod change_set_mvs;
pub mod dal_wrapper;
pub mod force_change_set_response;
pub mod index;
pub mod nats_multiplexer;
pub mod tracking;
pub mod workspace_permissions;

pub use edda_client::{
    ClientError as EddaClientError,
    EddaClient,
};

/// CRDT broadcast group type, moved here because it's used in AppState
pub type BroadcastGroups = Arc<Mutex<HashMap<String, Arc<y_sync::net::BroadcastGroup>>>>;

#[macro_export]
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
