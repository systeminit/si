pub mod protobuf {
    tonic::include_proto!("si.ssh_key");
}

//pub mod authorize;
pub mod error;
pub mod migrate;
pub mod model;
mod serde_enum;
pub mod service;

pub use migrate::migrate;
pub use protobuf::ssh_key_server::SshKeyServer;
pub use service::Service;
