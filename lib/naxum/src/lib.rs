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

// FIXME(nick): experimentation in not requiring "R: MessageHead" and "Message<R>".
mod generic;

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
        serve,
        serve_with_incoming_limit,
    },
    service_ext::ServiceExt,
};

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
