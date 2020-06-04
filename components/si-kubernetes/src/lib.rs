pub mod protobuf {
    tonic::include_proto!("si.kubernetes");
}

pub mod agent;
pub mod gen;
pub mod kubectl;
pub mod model;
