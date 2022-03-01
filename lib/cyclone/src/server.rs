pub use axum::extract::ws::Message as WebSocketMessage;

pub use config::{Config, ConfigBuilder, ConfigError, IncomingStream};
pub use decryption_key::{DecryptionKey, DecryptionKeyError};
pub use routes::routes;
pub use server::Server;
pub use uds::{UdsIncomingStream, UdsIncomingStreamError};

mod config;
mod decryption_key;
pub mod execution;
pub(crate) mod extract;
mod handlers;
mod request;
mod routes;
mod server;
pub(crate) mod tower;
mod uds;
pub mod watch;

use chrono::Utc;

pub fn timestamp() -> u64 {
    u64::try_from(std::cmp::max(Utc::now().timestamp(), 0)).expect("timestamp not be negative")
}
