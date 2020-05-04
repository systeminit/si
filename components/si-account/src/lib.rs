pub mod protobuf {
    tonic::include_proto!("si.account");
}

pub mod gen;

pub mod authorize;
pub mod error;
pub mod model;

pub use protobuf::{BillingAccount, Integration, IntegrationService, Workspace};
