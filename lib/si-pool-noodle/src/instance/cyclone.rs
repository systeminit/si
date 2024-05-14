//! Cyclone implementations of [`Instance`][`super::Instance`].

pub use local_http::{
    LocalHttpInstance, LocalHttpInstanceError, LocalHttpInstanceSpec, LocalHttpInstanceSpecBuilder,
    LocalHttpSocketStrategy,
};
pub use local_uds::{
    LocalUdsInstance, LocalUdsInstanceError, LocalUdsInstanceSpec, LocalUdsInstanceSpecBuilder,
    LocalUdsRuntimeStrategy, LocalUdsSocketStrategy,
};

mod local_http;
mod local_uds;
