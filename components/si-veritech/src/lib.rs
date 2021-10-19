#[macro_use]
mod cfg;
pub mod telemetry;

pub use si_cyclone::resolver_function::*;

cfg_feature! {
    #![feature = "server"]
    pub mod server;
    pub use server::{start, VeritechServerError};
}

cfg_feature! {
    #![feature = "server"]
    pub mod client;
}
