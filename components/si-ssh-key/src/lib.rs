pub mod ssh_key {
    tonic::include_proto!("ssh_key");
}

pub mod agent;
pub mod data;
pub mod error;
pub mod service;
pub mod settings;
