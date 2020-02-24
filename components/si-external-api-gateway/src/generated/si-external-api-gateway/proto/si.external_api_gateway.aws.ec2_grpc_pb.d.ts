// GENERATED CODE -- DO NOT EDIT!

// package: si.external_api_gateway.aws.ec2
// file: si-external-api-gateway/proto/si.external_api_gateway.aws.ec2.proto

import * as si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb from "../../si-external-api-gateway/proto/si.external_api_gateway.aws.ec2_pb";
import * as grpc from "grpc";

interface IEc2Service extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  createKeyPair: grpc.MethodDefinition<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairRequest, si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairReply>;
  describeKeyPairs: grpc.MethodDefinition<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsRequest, si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsReply>;
}

export const Ec2Service: IEc2Service;

export class Ec2Client extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  createKeyPair(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairRequest, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairReply>): grpc.ClientUnaryCall;
  createKeyPair(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairReply>): grpc.ClientUnaryCall;
  createKeyPair(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairReply>): grpc.ClientUnaryCall;
  describeKeyPairs(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsRequest, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsReply>): grpc.ClientUnaryCall;
  describeKeyPairs(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsReply>): grpc.ClientUnaryCall;
  describeKeyPairs(argument: si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<si_external_api_gateway_proto_si_external_api_gateway_aws_ec2_pb.DescribeKeyPairsReply>): grpc.ClientUnaryCall;
}
