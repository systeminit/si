pub mod change_set;
pub mod component;
pub mod diagram;
pub mod fix;
pub mod func;
pub mod provider;
pub mod qualification;
pub mod schema;
pub mod secret;
pub mod session;
pub mod signup;
pub mod status;
pub mod workflow;
pub mod ws;

/// A module containing test routes for integration testing.
#[cfg(debug_assertions)]
pub mod test;

/// A module containing dev routes for local development only.
#[cfg(debug_assertions)]
pub mod dev;
