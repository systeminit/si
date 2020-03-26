import AWS from "aws-sdk";
import { logger } from "@/logger";
import { Server, ServerUnaryCall, sendUnaryData } from "grpc";
import { EKSService } from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.eks_grpc_pb";
import {
  Bool,
  BoolMap,
  Certificate,
  Cluster,
  CreateClusterRequest,
  CreateClusterReply,
  CreateNodegroupRequest,
  CreateNodegroupReply,
  DescribeClusterRequest,
  DescribeClusterReply,
  DescribeNodegroupRequest,
  DescribeNodegroupReply,
  Logging,
  Label,
  Nodegroup,
  NodegroupHealth,
  NodegroupResources,
  NodegroupScalingConfig,
  RemoteAccessConfig,
  Tag,
  VpcConfigRequest,
  VpcConfigResponse,
  Error as PError,
} from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.eks_pb";
import { Err, Ok, Result } from "@usefultools/monads";

enum Error {
  MissingClusterName,
  MissingNodegroupName,
  MissingSubnets,
  MissingNodeRole,
}

type ErrorString = keyof typeof Error;

function toLabelsMap(input: Array<Label>): AWS.EKS.Types.labelsMap {
  const labels = {};
  for (const label of input) {
    labels[label.getKey()] = label.getValue();
  }

  return labels;
}

function toTagMap(input: Array<Tag>): AWS.EKS.Types.TagMap {
  const tags = {};
  for (const tag of input) {
    tags[tag.getKey()] = tag.getValue();
  }

  return tags;
}

function toBoolean(input: BoolMap[keyof BoolMap]): boolean | undefined {
  switch (input) {
    case Bool.TRUE:
      return true;
    case Bool.FALSE:
      return false;
    case Bool.BOOL_UNKNOWN:
      return undefined;
    default:
      return undefined;
  }
}

function toBool(input: boolean): BoolMap[keyof BoolMap] {
  if (input) {
    return Bool.TRUE;
  } else {
    return Bool.FALSE;
  }
}

function toVpcConfigRequest(
  input: VpcConfigRequest,
): AWS.EKS.Types.VpcConfigRequest {
  let resourcesVpcConfig: AWS.EKS.Types.VpcConfigRequest;
  if (input === undefined) {
    resourcesVpcConfig = {};
  } else {
    resourcesVpcConfig = {
      subnetIds: input.getSubnetIdsList(),
      securityGroupIds: input.getSecurityGroupIdsList(),
    };

    const endpointPublicAccess = toBoolean(input.getEndpointPublicAccess());
    if (endpointPublicAccess !== undefined) {
      resourcesVpcConfig.endpointPublicAccess = endpointPublicAccess;
    }
    const endpointPrivateAccess = toBoolean(input.getEndpointPrivateAccess());
    if (endpointPrivateAccess !== undefined) {
      resourcesVpcConfig.endpointPrivateAccess = endpointPrivateAccess;
    }
  }

  return resourcesVpcConfig;
}

function toLoggingRequest(input: Logging): AWS.EKS.Types.Logging {
  let logging;
  if (input === undefined) {
    logging = {
      clusterLogging: [],
    };
  } else {
    logging = {
      clusterLogging: input.getClusterLoggingList().map(input => {
        return {
          types: input.getTypesList(),
          enabled: input.getEnabled(),
        };
      }),
    };
  }

  return logging;
}

function toCreateClusterRequest(
  request: CreateClusterRequest,
): AWS.EKS.Types.CreateClusterRequest {
  return {
    name: request.getName(),
    version: request.getVersion(),
    roleArn: request.getRoleArn(),
    resourcesVpcConfig: toVpcConfigRequest(request.getResourcesVpcConfig()),
    logging: toLoggingRequest(request.getLogging()),
    clientRequestToken: request.getClientRequestToken(),
    tags: toTagMap(request.getTagsList()),
  };
}

function toNodegroupScalingConfigRequest(
  input: NodegroupScalingConfig,
): AWS.EKS.Types.NodegroupScalingConfig | undefined {
  if (input === undefined) {
    return undefined;
  }

  const output: AWS.EKS.Types.NodegroupScalingConfig = {};

  const minSize = input.getMinSize();
  if (minSize != 0) {
    output.minSize = minSize;
  }
  const maxSize = input.getMaxSize();
  if (maxSize != 0) {
    output.maxSize = maxSize;
  }
  const desiredSize = input.getDesiredSize();
  if (desiredSize != 0) {
    output.desiredSize = desiredSize;
  }

  if (Object.entries(output).length > 0) {
    return output;
  } else {
    return undefined;
  }
}

function toRemoteAccessConfigRequest(
  input: RemoteAccessConfig,
): AWS.EKS.Types.RemoteAccessConfig {
  if (input === undefined) {
    return undefined;
  }

  const output: AWS.EKS.Types.RemoteAccessConfig = {};

  // TODO(fnichol): ec2SshKey is required, but might come in as an empty
  // string. Is this the point where we introduce error/throw'ing?
  output.ec2SshKey = input.getEc2SshKey();
  const sourceSecurityGroups = input.getSourcesecuritygroupsList();
  if (sourceSecurityGroups.length > 0) {
    output.sourceSecurityGroups = sourceSecurityGroups;
  }

  return output;
}

function failCreateNodegroup(
  callback: sendUnaryData<CreateNodegroupReply>,
  e: { code?: string; message: string },
): void {
  const error = new PError();
  if (e.code !== undefined) {
    error.setCode(e.code);
  }
  error.setMessage(e.message);

  const reply = new CreateNodegroupReply();
  reply.setError(error);

  callback(null, reply);
}

function toCreateNodegroupRequest(
  input: CreateNodegroupRequest,
): Result<AWS.EKS.Types.CreateNodegroupRequest, Error> {
  const clusterName = input.getClusterName();
  if (clusterName.length == 0) {
    return Err(Error.MissingClusterName);
  }
  const nodegroupName = input.getNodegroupName();
  if (nodegroupName.length == 0) {
    return Err(Error.MissingNodegroupName);
  }
  const subnets = input.getSubnetsList();
  if (subnets.length == 0) {
    return Err(Error.MissingSubnets);
  }
  const nodeRole = input.getNodeRole();
  if (nodeRole.length == 0) {
    return Err(Error.MissingNodeRole);
  }

  const output: AWS.EKS.Types.CreateNodegroupRequest = {
    clusterName,
    nodegroupName,
    subnets,
    nodeRole,
  };

  const scalingConfig = toNodegroupScalingConfigRequest(
    input.getScalingConfig(),
  );
  if (scalingConfig !== undefined) {
    output.scalingConfig = scalingConfig;
  }
  const diskSize = input.getDiskSize();
  if (diskSize != 0) {
    output.diskSize = diskSize;
  }
  const instanceTypes = input
    .getInstanceTypesList()
    .filter(it => it.length > 0);
  if (instanceTypes.length > 0) {
    output.instanceTypes = instanceTypes;
  }
  const amiType = input.getAmiType();
  if (amiType.length > 0) {
    output.amiType = amiType;
  }
  const remoteAccess = toRemoteAccessConfigRequest(input.getRemoteAccess());
  if (remoteAccess !== undefined) {
    output.remoteAccess = remoteAccess;
  }
  const labels = toLabelsMap(input.getLabelsList());
  if (Object.keys(labels).length > 0) {
    output.labels = labels;
  }
  const tags = toTagMap(input.getTagsList());
  if (Object.keys(tags).length > 0) {
    output.tags = tags;
  }
  const clientRequestToken = input.getClientRequestToken();
  if (clientRequestToken.length > 0) {
    output.clientRequestToken = clientRequestToken;
  }
  const version = input.getVersion();
  if (version.length > 0) {
    output.version = version;
  }
  const releaseVersion = input.getReleaseVersion();
  if (releaseVersion.length > 0) {
    output.releaseVersion = releaseVersion;
  }

  return Ok(output);
}

function toCertificate(ctx: AWS.EKS.Types.Certificate): Certificate {
  const certificateAuthority = new Certificate();
  certificateAuthority.setData(ctx.data);

  return certificateAuthority;
}

function toVpcConfigResponse(
  ctx: AWS.EKS.Types.VpcConfigResponse,
): VpcConfigResponse {
  const resourcesVpcConfig = new VpcConfigResponse();
  resourcesVpcConfig.setClusterSecurityGroupId(ctx.clusterSecurityGroupId);
  resourcesVpcConfig.setEndpointPrivateAccess(
    toBool(ctx.endpointPrivateAccess),
  );
  resourcesVpcConfig.setEndpointPublicAccess(toBool(ctx.endpointPublicAccess));
  resourcesVpcConfig.setPublicAccessCidrsList(ctx.publicAccessCidrs);
  resourcesVpcConfig.setSecurityGroupIdsList(ctx.securityGroupIds);
  resourcesVpcConfig.setSubnetIdsList(ctx.subnetIds);
  resourcesVpcConfig.setVpcId(ctx.vpcId);

  return resourcesVpcConfig;
}

function toLabelsList(input: AWS.EKS.Types.labelsMap): Array<Label> {
  return Object.entries(input).map(([key, value]) => {
    const label = new Label();
    label.setKey(key);
    label.setValue(value);

    return label;
  });
}

function toTagsList(input: AWS.EKS.Types.TagMap): Array<Tag> {
  const tags = Object.entries(input).map(([key, value]) => {
    const tag = new Tag();
    tag.setKey(key);
    tag.setValue(value);

    return tag;
  });

  return tags;
}

function toNodegroupHealth(
  input: AWS.EKS.Types.NodegroupHealth,
): NodegroupHealth {
  const health = new NodegroupHealth();
  health.setIssuesList(
    input.issues.map(input => {
      const issue = new NodegroupHealth.Issue();
      issue.setCode(input.code);
      issue.setMessage(input.message);
      issue.setResourceIdsList(input.resourceIds);

      return issue;
    }),
  );

  return health;
}

function toRemoteAccessConfig(
  input: AWS.EKS.Types.RemoteAccessConfig,
): RemoteAccessConfig {
  const remoteAccess = new RemoteAccessConfig();
  remoteAccess.setEc2SshKey(input.ec2SshKey);
  remoteAccess.setSourcesecuritygroupsList(input.sourceSecurityGroups);

  return remoteAccess;
}

function toAutoScalingGroupsList(
  input: AWS.EKS.Types.AutoScalingGroupList,
): Array<NodegroupResources.AutoScalingGroup> {
  return input.map(input => {
    const output = new NodegroupResources.AutoScalingGroup();
    output.setName(input.name);

    return output;
  });
}

function toNodegroupResources(
  input: AWS.EKS.Types.NodegroupResources,
): NodegroupResources {
  const output = new NodegroupResources();

  const autoScalingGroups = input.autoScalingGroups;
  if (autoScalingGroups !== undefined) {
    output.setAutoScalingGroupsList(toAutoScalingGroupsList(autoScalingGroups));
  }
  output.setRemoteAccessSecurityGroup(input.remoteAccessSecurityGroup);

  return output;
}

function toNodegroupScalingConfig(
  input: AWS.EKS.Types.NodegroupScalingConfig,
): NodegroupScalingConfig {
  const scalingConfig = new NodegroupScalingConfig();
  scalingConfig.setDesiredSize(input.desiredSize);
  scalingConfig.setMaxSize(input.maxSize);
  scalingConfig.setMinSize(input.minSize);

  return scalingConfig;
}

function toNodegroup(input: AWS.EKS.Types.Nodegroup): Nodegroup {
  const nodegroup = new Nodegroup();
  nodegroup.setAmiType(input.amiType);
  nodegroup.setClusterName(input.clusterName);
  nodegroup.setCreatedAt(input.createdAt.toUTCString());
  nodegroup.setDiskSize(input.diskSize);
  nodegroup.setHealth(toNodegroupHealth(input.health));
  nodegroup.setInstanceTypesList(input.instanceTypes);
  const labels = input.labels;
  if (labels !== undefined) {
    nodegroup.setLabelsList(toLabelsList(labels));
  }
  nodegroup.setModifiedAt(input.modifiedAt.toUTCString());
  nodegroup.setNodegroupArn(input.nodegroupArn);
  nodegroup.setNodegroupName(input.nodegroupName);
  nodegroup.setNodeRole(input.nodeRole);
  nodegroup.setReleaseVersion(input.releaseVersion);
  nodegroup.setRemoteAccess(toRemoteAccessConfig(input.remoteAccess));
  const resources = input.resources;
  if (resources !== undefined) {
    nodegroup.setResources(toNodegroupResources(resources));
  }
  nodegroup.setScalingConfig(toNodegroupScalingConfig(input.scalingConfig));
  nodegroup.setStatus(input.status);
  nodegroup.setSubnetsList(input.subnets);
  nodegroup.setTagsList(toTagsList(input.tags));
  nodegroup.setVersion(input.version);

  return nodegroup;
}

function toCluster(ctx: AWS.EKS.Types.Cluster): Cluster {
  const oidc = new Cluster.Identity.Oidc();
  oidc.setIssuer(ctx.identity?.oidc?.issuer);

  const identity = new Cluster.Identity();
  identity.setOidc(oidc);

  const clusterLogging = ctx.logging.clusterLogging.map(ls => {
    const logSetup = new Logging.LogSetup();
    logSetup.setEnabled(ls.enabled);
    logSetup.setTypesList(ls.types);

    return logSetup;
  });

  const logging = new Logging();
  logging.setClusterLoggingList(clusterLogging);

  const tags = Object.entries(ctx.tags).map(([key, value]) => {
    const tag = new Tag();
    tag.setKey(key);
    tag.setValue(value);

    return tag;
  });

  const cluster = new Cluster();
  cluster.setArn(ctx.arn);
  cluster.setCertificateAuthority(toCertificate(ctx.certificateAuthority));
  cluster.setClientRequestToken(ctx.clientRequestToken);
  cluster.setCreatedAt(ctx.createdAt.toUTCString());
  cluster.setEndpoint(ctx.endpoint);
  cluster.setIdentity(identity);
  cluster.setLogging(logging);
  cluster.setName(ctx.name);
  cluster.setPlatformVersion(ctx.platformVersion);
  cluster.setResourcesVpcConfig(toVpcConfigResponse(ctx.resourcesVpcConfig));
  cluster.setRoleArn(ctx.roleArn);
  cluster.setStatus(ctx.status);
  cluster.setTagsList(tags);
  cluster.setVersion(ctx.version);

  return cluster;
}

function toCreateClusterReply(
  response: AWS.EKS.Types.CreateClusterResponse,
): CreateClusterReply {
  const reply = new CreateClusterReply();
  reply.setCluster(toCluster(response.cluster));

  return reply;
}

function toCreateNodegroupReply(
  response: AWS.EKS.Types.CreateNodegroupResponse,
): CreateNodegroupReply {
  const reply = new CreateNodegroupReply();
  reply.setNodegroup(toNodegroup(response.nodegroup));

  return reply;
}

function toDescribeClusterRequest(
  request: DescribeClusterRequest,
): AWS.EKS.Types.DescribeClusterRequest {
  return {
    name: request.getName(),
  };
}

function toDescribeClusterReply(
  response: AWS.EKS.Types.DescribeClusterResponse,
): DescribeClusterReply {
  const reply = new DescribeClusterReply();
  reply.setCluster(toCluster(response.cluster));

  return reply;
}

function toDescribeNodegroupRequest(
  request: DescribeNodegroupRequest,
): AWS.EKS.Types.DescribeNodegroupRequest {
  return {
    clusterName: request.getClusterName(),
    nodegroupName: request.getNodegroupName(),
  };
}

function toDescribeNodegroupReply(
  response: AWS.EKS.Types.DescribeNodegroupResponse,
): DescribeNodegroupReply {
  const reply = new DescribeNodegroupReply();
  reply.setNodegroup(toNodegroup(response.nodegroup));

  return reply;
}

class AwsEks {
  constructor() {
    // These will be used as callbacks - so eventually, they are called in a way
    // that has them totally detached from their original context (in this case,
    // the instance of our class here.) This dirty mojo makes that not happen,
    // and ensures that the callbacks have reasonable encapsulation.
    this.createCluster = this.createCluster.bind(this);
    this.createNodegroup = this.createNodegroup.bind(this);
    this.describeCluster = this.describeCluster.bind(this);
    this.describeNodegroup = this.describeNodegroup.bind(this);
  }

  addToServer(server: Server): void {
    logger.log("info", "Adding AWS EKS");
    server.addService(EKSService, {
      createCluster: this.createCluster,
      createNodegroup: this.createNodegroup,
      describeCluster: this.describeCluster,
      describeNodegroup: this.describeNodegroup,
    });
  }

  async createCluster(
    call: ServerUnaryCall<CreateClusterRequest>,
    callback: sendUnaryData<CreateClusterReply>,
  ): Promise<void> {
    const awsLogger = {
      log(foo: string): void {
        logger.info(foo);
      },
    };
    const eksClient = new AWS.EKS({ logger: awsLogger, region: "us-east-2" });
    const request = toCreateClusterRequest(call.request);
    let response;

    try {
      response = await eksClient.createCluster(request).promise();
    } catch (err) {
      const error = new PError();
      error.setCode(err.code);
      error.setMessage(err.message);

      const reply = new CreateClusterReply();
      reply.setError(error);

      callback(null, reply);
      return;
    }

    const reply = toCreateClusterReply(response);

    callback(null, reply);
  }

  async createNodegroup(
    call: ServerUnaryCall<CreateNodegroupRequest>,
    callback: sendUnaryData<CreateNodegroupReply>,
  ): Promise<void> {
    const awsLogger = {
      log(foo: string): void {
        logger.info(foo);
      },
    };
    const eksClient = new AWS.EKS({ logger: awsLogger, region: "us-east-2" });

    const result = toCreateNodegroupRequest(call.request);
    if (result.is_err()) {
      failCreateNodegroup(callback, {
        message: Error[result.unwrap_err()],
      });
      return;
    }
    const request = result.unwrap();

    let response;
    try {
      response = await eksClient.createNodegroup(request).promise();
    } catch (e) {
      failCreateNodegroup(callback, e);
      return;
    }

    const reply = toCreateNodegroupReply(response);

    callback(null, reply);
  }

  async describeCluster(
    call: ServerUnaryCall<DescribeClusterRequest>,
    callback: sendUnaryData<DescribeClusterReply>,
  ): Promise<void> {
    const awsLogger = {
      log(foo: string): void {
        logger.info(foo);
      },
    };
    const eksClient = new AWS.EKS({ logger: awsLogger, region: "us-east-2" });
    const request = toDescribeClusterRequest(call.request);
    let response;

    try {
      response = await eksClient.describeCluster(request).promise();
    } catch (err) {
      const error = new PError();
      error.setCode(err.code);
      error.setMessage(err.message);

      const reply = new DescribeClusterReply();
      reply.setError(error);

      callback(null, reply);
      return;
    }

    const reply = toDescribeClusterReply(response);

    callback(null, reply);
  }

  async describeNodegroup(
    call: ServerUnaryCall<DescribeNodegroupRequest>,
    callback: sendUnaryData<DescribeNodegroupReply>,
  ): Promise<void> {
    const awsLogger = {
      log(foo: string): void {
        logger.info(foo);
      },
    };
    const eksClient = new AWS.EKS({ logger: awsLogger, region: "us-east-2" });
    const request = toDescribeNodegroupRequest(call.request);
    let response;

    try {
      response = await eksClient.describeNodegroup(request).promise();
    } catch (err) {
      const error = new PError();
      error.setCode(err.code);
      error.setMessage(err.message);

      const reply = new DescribeNodegroupReply();
      reply.setError(error);

      callback(null, reply);
      return;
    }

    const reply = toDescribeNodegroupReply(response);

    callback(null, reply);
  }
}

export const AwsEksService = new AwsEks();
