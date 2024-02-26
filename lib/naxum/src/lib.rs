#[macro_use]
pub(crate) mod macros;

mod cancellation;
mod error;
pub mod error_handling;
pub mod extract;
pub mod handler;
mod make_service;
mod message;
pub mod response;
pub mod serve;
mod service_ext;

pub use self::cancellation::wait_on_cancelled;
pub use self::error::Error;
pub use self::message::MessageHead;
pub use self::serve::serve;
pub use self::service_ext::ServiceExt;
pub use async_trait::async_trait;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
