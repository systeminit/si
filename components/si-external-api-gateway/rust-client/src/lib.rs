use std::env;

pub mod protobuf {
    tonic::include_proto!("si.external_api_gateway");
    pub mod aws {
        pub mod ec2 {
            tonic::include_proto!("si.external_api_gateway.aws.ec2");
        }
        pub mod eks {
            tonic::include_proto!("si.external_api_gateway.aws.eks");
        }
    }
}

pub mod aws {
    pub mod ec2 {
        pub use crate::protobuf::aws::ec2::{
            ec2_client::Ec2Client, CreateKeyPairReply, CreateKeyPairRequest, DescribeKeyPairsReply,
            DescribeKeyPairsRequest, Error, Filter,
        };
        pub use crate::protobuf::Context;
    }
    pub mod eks {
        pub use crate::protobuf::aws::eks::{
            eks_client::EksClient, logging, Bool, Certificate, Cluster, CreateClusterReply,
            CreateClusterRequest, CreateNodegroupReply, CreateNodegroupRequest,
            DescribeClusterReply, DescribeClusterRequest, DescribeNodegroupReply,
            DescribeNodegroupRequest, Error, Label, Logging, Nodegroup, NodegroupHealth,
            NodegroupResources, NodegroupScalingConfig, RemoteAccessConfig, Tag, VpcConfigRequest,
            VpcConfigResponse,
        };
        pub use crate::protobuf::Context;
        use std::convert::TryFrom;

        #[derive(thiserror::Error, Debug)]
        #[error("unknown Bool value")]
        pub struct UnknownBoolError(());

        #[derive(thiserror::Error, Debug)]
        #[error("invalid Bool value: {0}")]
        pub struct InvalidBoolError(i32);

        impl TryFrom<Bool> for bool {
            type Error = UnknownBoolError;

            fn try_from(value: Bool) -> std::result::Result<Self, Self::Error> {
                match value {
                    Bool::Unknown => Err(UnknownBoolError(())),
                    Bool::True => Ok(true),
                    Bool::False => Ok(false),
                }
            }
        }

        impl TryFrom<i32> for Bool {
            type Error = InvalidBoolError;

            fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
                Self::from_i32(value).ok_or_else(|| InvalidBoolError(value))
            }
        }

        impl From<bool> for Bool {
            fn from(value: bool) -> Self {
                if value {
                    Self::True
                } else {
                    Self::False
                }
            }
        }
    }
}

pub fn gateway_url() -> String {
    match env::var_os("SI_EXTERNAL_API_GATEWAY_URL") {
        Some(val) => val
            .into_string()
            .unwrap_or("http://localhost:4001".to_string()),
        None => "http://localhost:4001".to_string(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
