// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var si$external$api$gateway_proto_si_external_api_gateway_aws_ec2_pb = require('../../si-external-api-gateway/proto/si.external_api_gateway.aws.ec2_pb.js');
var si$external$api$gateway_proto_si_external_api_gateway_pb = require('../../si-external-api-gateway/proto/si.external_api_gateway_pb.js');

function serialize_si_external_api_gateway_aws_ec2_CreateKeyPairReply(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairReply)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.ec2.CreateKeyPairReply');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_ec2_CreateKeyPairReply(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairReply.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_si_external_api_gateway_aws_ec2_CreateKeyPairRequest(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairRequest)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.ec2.CreateKeyPairRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_ec2_CreateKeyPairRequest(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairRequest.deserializeBinary(new Uint8Array(buffer_arg));
}


var Ec2Service = exports.Ec2Service = {
  createKeyPair: {
    path: '/si.external_api_gateway.aws.ec2.Ec2/CreateKeyPair',
    requestStream: false,
    responseStream: false,
    requestType: si$external$api$gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairRequest,
    responseType: si$external$api$gateway_proto_si_external_api_gateway_aws_ec2_pb.CreateKeyPairReply,
    requestSerialize: serialize_si_external_api_gateway_aws_ec2_CreateKeyPairRequest,
    requestDeserialize: deserialize_si_external_api_gateway_aws_ec2_CreateKeyPairRequest,
    responseSerialize: serialize_si_external_api_gateway_aws_ec2_CreateKeyPairReply,
    responseDeserialize: deserialize_si_external_api_gateway_aws_ec2_CreateKeyPairReply,
  },
};

exports.Ec2Client = grpc.makeGenericClientConstructor(Ec2Service);
