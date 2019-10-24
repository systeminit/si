pub mod protobuf {
    tonic::include_proto!("si.account");
}

pub mod authorize;
pub mod error;
pub mod model;
pub mod service;
