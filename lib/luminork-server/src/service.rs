pub mod whoami;

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
