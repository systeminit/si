// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb = require('../../si-external-api-gateway/proto/si.external_api_gateway.aws.eks_pb.js');
var si$external$api$gateway_proto_si_external_api_gateway_pb = require('../../si-external-api-gateway/proto/si.external_api_gateway_pb.js');

function serialize_si_external_api_gateway_aws_eks_CreateClusterReply(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.CreateClusterReply');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_CreateClusterReply(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_si_external_api_gateway_aws_eks_CreateClusterRequest(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.CreateClusterRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_CreateClusterRequest(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest.deserializeBinary(new Uint8Array(buffer_arg));
}


var EKSService = exports.EKSService = {
  createCluster: {
    path: '/si.external_api_gateway.aws.eks.EKS/CreateCluster',
    requestStream: false,
    responseStream: false,
    requestType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterRequest,
    responseType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateClusterReply,
    requestSerialize: serialize_si_external_api_gateway_aws_eks_CreateClusterRequest,
    requestDeserialize: deserialize_si_external_api_gateway_aws_eks_CreateClusterRequest,
    responseSerialize: serialize_si_external_api_gateway_aws_eks_CreateClusterReply,
    responseDeserialize: deserialize_si_external_api_gateway_aws_eks_CreateClusterReply,
  },
};

exports.EKSClient = grpc.makeGenericClientConstructor(EKSService);
