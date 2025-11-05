use thiserror::Error;

pub mod component;
mod parser;
mod query;

pub use query::{
    SearchQuery,
    SearchTerm,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("frigg error: {0}")]
    Frigg(#[from] frigg::Error),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    // TODO(jkeiser) this should be inside frigg, no?
    #[error("mv item not found: {0}, {1}, {2} (kind, id, checksum)")]
    MvNotFound(String, String, String), // kind, id, checksum
    #[error(
        "The search parser stopped unexpectedly at position {position} in query '{query_string}'"
    )]
    ParserFailed {
        query_string: String,
        position: usize,
    },
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

pub type Result<T> = std::result::Result<T, Error>;
