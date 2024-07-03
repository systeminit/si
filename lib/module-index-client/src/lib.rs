pub mod client;
pub mod types;

pub use client::IndexClient;
pub use types::{
    ExtraMetadata, FuncMetadata, IndexClientError, IndexClientResult, ModuleDetailsResponse,
};

pub const DEFAULT_URL: &str = "http://localhost:5157";
pub const MODULE_BUNDLE_FIELD_NAME: &str = "module_bundle";
pub const MODULE_BASED_ON_HASH_FIELD_NAME: &str = "based_on_hash";
pub const MODULE_SCHEMA_ID_FIELD_NAME: &str = "schema_id";
