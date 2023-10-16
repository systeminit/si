#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]

pub mod canonical_file;
pub mod option;
pub mod result;
pub mod string;

pub use canonical_file::{CanonicalFile, CanonicalFileError};
pub use option::OptionExt;
pub use result::ResultExt;
pub use string::SensitiveString;
