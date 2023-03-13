//! This module contains "long-running" [tokio](https://tokio.rs) tasks that run alongside
//! SI binaries that are dependent on the [`dal`](crate).

// This modules should remain private! Add "pub use" statements to use their contents.
mod resource_scheduler;
mod status_receiver;

pub use resource_scheduler::{ResourceScheduler, ResourceSchedulerError};
pub use status_receiver::client::StatusReceiverClient;
pub use status_receiver::{StatusReceiver, StatusReceiverError, StatusReceiverRequest};
