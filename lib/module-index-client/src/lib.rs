pub mod client;
pub mod types;

pub use client::IndexClient;
pub use types::{FuncMetadata, IndexClientError, IndexClientResult, ModuleDetailsResponse};

pub const DEFAULT_URL: &str = "http://localhost:5157";
