#[macro_use]
pub(crate) mod macros;

pub mod body;
mod cancellation;
mod error;
pub mod error_handling;
pub mod extract;
pub mod handler;
mod json;
mod make_service;
mod message;
pub mod middleware;
pub mod response;
pub mod serve;
mod service_ext;

pub use self::cancellation::wait_on_cancelled;
pub use self::error::Error;
pub use self::json::Json;
pub use self::make_service::IntoMakeService;
pub use self::message::{Extensions, Head, HeadRef, Message, MessageHead};
pub use self::serve::{
    serve, serve_with_incoming_limit, serve_with_incoming_limit_and_force_reconnect_sender,
};
pub use self::service_ext::ServiceExt;

pub use async_nats::StatusCode;
pub use async_trait::async_trait;
pub use tower::ServiceBuilder;
pub use tower::ServiceExt as TowerServiceExt;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
