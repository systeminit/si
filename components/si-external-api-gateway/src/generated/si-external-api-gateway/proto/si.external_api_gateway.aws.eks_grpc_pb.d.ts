// GENERATED CODE -- DO NOT EDIT!

// package: si.external_api_gateway.aws.eks
// file: si-external-api-gateway/proto/si.external_api_gateway.aws.eks.proto

import * as si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb from "../../si-external-api-gateway/proto/si.external_api_gateway.aws.eks_pb";
import * as grpc from "grpc";

interface IEKSService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  createCluster: grpc.MethodDefinition<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>;
  createNodegroup: grpc.MethodDefinition<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupRequest, si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupReply>;
  describeCluster: grpc.MethodDefinition<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterRequest, si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterReply>;
  describeNodegroup: grpc.MethodDefinition<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupRequest, si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupReply>;
}

export const EKSService: IEKSService;

export class EKSClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  createCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>): grpc.ClientUnaryCall;
  createCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>): grpc.ClientUnaryCall;
  createCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>): grpc.ClientUnaryCall;
  createNodegroup(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupRequest, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupReply>): grpc.ClientUnaryCall;
  createNodegroup(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupReply>): grpc.ClientUnaryCall;
  createNodegroup(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupReply>): grpc.ClientUnaryCall;
  describeCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterRequest, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterReply>): grpc.ClientUnaryCall;
  describeCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterReply>): grpc.ClientUnaryCall;
  describeCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterReply>): grpc.ClientUnaryCall;
  describeNodegroup(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupRequest, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupReply>): grpc.ClientUnaryCall;
  describeNodegroup(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupReply>): grpc.ClientUnaryCall;
  describeNodegroup(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupReply>): grpc.ClientUnaryCall;
}
