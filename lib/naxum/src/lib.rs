#[macro_use]
pub(crate) mod macros;

pub mod body;
mod cancellation;
mod error;
pub mod error_handling;
pub mod extract;
pub mod fair;
pub mod handler;
mod json;
mod make_service;
mod message;
pub mod middleware;
pub mod response;
pub mod serve;
mod serve_builder;
mod service_ext;

pub use async_nats::StatusCode;
pub use async_trait::async_trait;
pub use tower::{
    ServiceBuilder,
    ServiceExt as TowerServiceExt,
};

pub use self::{
    cancellation::wait_on_cancelled,
    error::Error,
    json::Json,
    make_service::IntoMakeService,
    message::{
        Extensions,
        FromPartsError,
        Head,
        HeadRef,
        Message,
        MessageHead,
    },
    serve::{
        SemaphoreMode,
        serve,
        serve_with_external_semaphore,
        serve_with_incoming_limit,
    },
    serve_builder::ServeBuilder,
    service_ext::ServiceExt,
};

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
