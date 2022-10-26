#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(clippy::missing_errors_doc)]

pub mod faktory;
pub use faktory::FaktoryConfig;

mod sensitive_string;
pub use sensitive_string::SensitiveString;

mod result_ext;
pub use result_ext::ResultExt;

mod option_ext;
pub use option_ext::OptionExt;
