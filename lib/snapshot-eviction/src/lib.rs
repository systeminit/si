pub mod config;
pub mod error;
pub mod evictor;

pub use config::SnapshotEvictionConfig;
pub use error::{
    SnapshotEvictionError,
    SnapshotEvictionResult,
};
pub use evictor::SnapshotEvictor;
