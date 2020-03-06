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

function serialize_si_external_api_gateway_aws_eks_CreateNodegroupReply(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupReply)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.CreateNodegroupReply');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_CreateNodegroupReply(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupReply.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_si_external_api_gateway_aws_eks_CreateNodegroupRequest(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupRequest)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.CreateNodegroupRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_CreateNodegroupRequest(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_si_external_api_gateway_aws_eks_DescribeClusterReply(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterReply)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.DescribeClusterReply');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_DescribeClusterReply(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterReply.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_si_external_api_gateway_aws_eks_DescribeClusterRequest(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterRequest)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.DescribeClusterRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_DescribeClusterRequest(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_si_external_api_gateway_aws_eks_DescribeNodegroupReply(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupReply)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.DescribeNodegroupReply');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_DescribeNodegroupReply(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupReply.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_si_external_api_gateway_aws_eks_DescribeNodegroupRequest(arg) {
  if (!(arg instanceof si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupRequest)) {
    throw new Error('Expected argument of type si.external_api_gateway.aws.eks.DescribeNodegroupRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_si_external_api_gateway_aws_eks_DescribeNodegroupRequest(buffer_arg) {
  return si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupRequest.deserializeBinary(new Uint8Array(buffer_arg));
}


// [API reference](https://docs.aws.amazon.com/eks/latest/APIReference/Welcome.html)
var EKSService = exports.EKSService = {
  // Creates an Amazon EKS control plane.
//
// The Amazon EKS control plane consists of control plane instances that run
// the Kubernetes software, such as etcd and the API server. The control
// plane runs in an account managed by AWS, and the Kubernetes API is exposed
// via the Amazon EKS API server endpoint. Each Amazon EKS cluster control
// plane is single-tenant and unique and runs on its own set of Amazon EC2
// instances.
//
// The cluster control plane is provisioned across multiple Availability
// Zones and fronted by an Elastic Load Balancing Network Load Balancer.
// Amazon EKS also provisions elastic network interfaces in your VPC subnets
// to provide connectivity from the control plane instances to the worker
// nodes (for example, to support kubectl exec, logs, and proxy data flows).
//
// Amazon EKS worker nodes run in your AWS account and connect to your
// cluster's control plane via the Kubernetes API server endpoint and a
// certificate file that is created for your cluster.
//
// You can use the endpointPublicAccess and endpointPrivateAccess parameters
// to enable or disable public and private access to your cluster's
// Kubernetes API server endpoint. By default, public access is enabled, and
// private access is disabled. For more information, see Amazon EKS Cluster
// Endpoint Access Control in the Amazon EKS User Guide .
//
// You can use the logging parameter to enable or disable exporting the
// Kubernetes control plane logs for your cluster to CloudWatch Logs. By
// default, cluster control plane logs aren't exported to CloudWatch Logs.
// For more information, see Amazon EKS Cluster Control Plane Logs in the
// Amazon EKS User Guide.
//
// Note: CloudWatch Logs ingestion, archive storage, and data scanning rates
// apply to exported control plane logs. For more information, see Amazon
// CloudWatch Pricing.
//
// Cluster creation typically takes between 10 and 15 minutes. After you
// create an Amazon EKS cluster, you must configure your Kubernetes tooling
// to communicate with the API server and launch worker nodes into your
// cluster.  For more information, see Managing Cluster Authentication and
// Launching Amazon EKS Worker Nodes in the Amazon EKS User Guide.
//
// [API Reference](https://docs.aws.amazon.com/eks/latest/APIReference/API_CreateCluster.html)
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
  // Creates a managed worker node group for an Amazon EKS cluster. You can
// only create a node group for your cluster that is equal to the current
// Kubernetes version for the cluster. All node groups are created with the
// latest AMI release version for the respective minor Kubernetes version of
// the cluster.
//
// An Amazon EKS managed node group is an Amazon EC2 Auto Scaling group and
// associated Amazon EC2 instances that are managed by AWS for an Amazon EKS
// cluster. Each node group uses a version of the Amazon EKS-optimized Amazon
// Linux 2 AMI. For more information, see Managed Node Groups in the Amazon
// EKS User Guide.
//
// [API Reference](https://docs.aws.amazon.com/eks/latest/APIReference/API_CreateNodegroup.html)
createNodegroup: {
    path: '/si.external_api_gateway.aws.eks.EKS/CreateNodegroup',
    requestStream: false,
    responseStream: false,
    requestType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupRequest,
    responseType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.CreateNodegroupReply,
    requestSerialize: serialize_si_external_api_gateway_aws_eks_CreateNodegroupRequest,
    requestDeserialize: deserialize_si_external_api_gateway_aws_eks_CreateNodegroupRequest,
    responseSerialize: serialize_si_external_api_gateway_aws_eks_CreateNodegroupReply,
    responseDeserialize: deserialize_si_external_api_gateway_aws_eks_CreateNodegroupReply,
  },
  // Returns descriptive information about an Amazon EKS cluster.
//
// The API server endpoint and certificate authority data returned by this
// operation are required for kubelet and kubectl to communicate with your
// Kubernetes API server. For more information, see Create a kubeconfig for
// Amazon EKS.
//
// Note: The API server endpoint and certificate authority data aren't
// available until the cluster reaches the `ACTIVE` state.
//
// [API Reference](https://docs.aws.amazon.com/eks/latest/APIReference/API_DescribeCluster.html)
describeCluster: {
    path: '/si.external_api_gateway.aws.eks.EKS/DescribeCluster',
    requestStream: false,
    responseStream: false,
    requestType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterRequest,
    responseType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeClusterReply,
    requestSerialize: serialize_si_external_api_gateway_aws_eks_DescribeClusterRequest,
    requestDeserialize: deserialize_si_external_api_gateway_aws_eks_DescribeClusterRequest,
    responseSerialize: serialize_si_external_api_gateway_aws_eks_DescribeClusterReply,
    responseDeserialize: deserialize_si_external_api_gateway_aws_eks_DescribeClusterReply,
  },
  // Returns descriptive information about an Amazon EKS node group.
//
// [API Reference](https://docs.aws.amazon.com/eks/latest/APIReference/API_DescribeNodegroup.html)
describeNodegroup: {
    path: '/si.external_api_gateway.aws.eks.EKS/DescribeNodegroup',
    requestStream: false,
    responseStream: false,
    requestType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupRequest,
    responseType: si$external$api$gateway_proto_si_external_api_gateway_aws_eks_pb.DescribeNodegroupReply,
    requestSerialize: serialize_si_external_api_gateway_aws_eks_DescribeNodegroupRequest,
    requestDeserialize: deserialize_si_external_api_gateway_aws_eks_DescribeNodegroupRequest,
    responseSerialize: serialize_si_external_api_gateway_aws_eks_DescribeNodegroupReply,
    responseDeserialize: deserialize_si_external_api_gateway_aws_eks_DescribeNodegroupReply,
  },
};

exports.EKSClient = grpc.makeGenericClientConstructor(EKSService);
