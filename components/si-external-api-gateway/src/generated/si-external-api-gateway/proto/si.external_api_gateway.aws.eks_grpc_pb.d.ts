// GENERATED CODE -- DO NOT EDIT!

// package: si.external_api_gateway.aws.eks
// file: si-external-api-gateway/proto/si.external_api_gateway.aws.eks.proto

import * as si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb from "../../si-external-api-gateway/proto/si.external_api_gateway.aws.eks_pb";
import * as grpc from "grpc";

interface IEKSService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  createCluster: grpc.MethodDefinition<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>;
}

export const EKSService: IEKSService;

export class EKSClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  createCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>): grpc.ClientUnaryCall;
  createCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>): grpc.ClientUnaryCall;
  createCluster(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply>): grpc.ClientUnaryCall;
}
