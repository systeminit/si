pub mod client;
pub mod types;

pub use client::IndexClient;
pub use types::{upload::UploadResponse, IndexClientError, IndexClientResult};

pub const DEFAULT_URL: &str = "http://localhost:5157";
