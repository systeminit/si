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
            eks_client::EksClient, logging, Certificate, Cluster, CreateClusterReply,
            CreateClusterRequest, CreateNodegroupReply, CreateNodegroupRequest,
            DescribeClusterReply, DescribeClusterRequest, DescribeNodegroupReply,
            DescribeNodegroupRequest, Error, Label, Logging, Nodegroup, NodegroupHealth,
            NodegroupResources, NodegroupScalingConfig, RemoteAccessConfig, Tag, VpcConfigRequest,
            VpcConfigResponse,
        };
        pub use crate::protobuf::Context;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
