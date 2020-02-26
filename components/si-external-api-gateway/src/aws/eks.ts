import AWS from "aws-sdk";

import { logger } from "@/logger";
import { Server, ServerUnaryCall, sendUnaryData } from "grpc";
import { EKSService } from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.eks_grpc_pb";
import {
  CreateClusterRequest,
  CreateClusterReply,
  Logging,
  Tag,
  Error as PError,
} from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.eks_pb";

function buildCreateClusterRequest(
  request: CreateClusterRequest,
): AWS.EKS.Types.CreateClusterRequest {
  let resourcesVpcConfig;
  if (request.getResourcesVpcConfig() === undefined) {
    resourcesVpcConfig = {};
  } else {
    resourcesVpcConfig = {
      subnetIds: request.getResourcesVpcConfig().getSubnetIdsList(),
      securityGroupIds: request
        .getResourcesVpcConfig()
        .getSecurityGroupIdsList(),
      endpointPublicAccess: request
        .getResourcesVpcConfig()
        .getEndpointPublicAccess(),
      endpointPrivateAccess: request
        .getResourcesVpcConfig()
        .getEndpointPrivateAccess(),
    };
  }

  let logging;
  if (request.getLogging() === undefined) {
    logging = {
      clusterLogging: [],
    };
  } else {
    logging = {
      clusterLogging: request
        .getLogging()
        .getClusterLoggingList()
        .map(logSetup => {
          return {
            types: logSetup.getTypesList(),
            enabled: logSetup.getEnabled(),
          };
        }),
    };
  }

  const tags = {};
  for (const tag of request.getTagsList()) {
    tags[tag.getKey()] = tag.getValue();
  }

  return {
    name: request.getName(),
    version: request.getVersion(),
    roleArn: request.getRoleArn(),
    resourcesVpcConfig,
    logging,
    clientRequestToken: request.getClientRequestToken(),
    tags,
  };
}

function toReplyClusterCertificate(
  ctx: AWS.EKS.Types.Certificate,
): CreateClusterReply.Cluster.Certificate {
  const certificateAuthority = new CreateClusterReply.Cluster.Certificate();
  certificateAuthority.setData(ctx.data);

  return certificateAuthority;
}

function toReplyClusterVpcConfigResponse(
  ctx: AWS.EKS.Types.VpcConfigResponse,
): CreateClusterReply.Cluster.VpcConfigResponse {
  const resourcesVpcConfig = new CreateClusterReply.Cluster.VpcConfigResponse();
  resourcesVpcConfig.setClusterSecurityGroupId(ctx.clusterSecurityGroupId);
  resourcesVpcConfig.setEndpointPrivateAccess(ctx.endpointPrivateAccess);
  resourcesVpcConfig.setEndpointPublicAccess(ctx.endpointPublicAccess);
  resourcesVpcConfig.setPublicAccessCidrsList(ctx.publicAccessCidrs);
  resourcesVpcConfig.setSecurityGroupIdsList(ctx.securityGroupIds);
  resourcesVpcConfig.setSubnetIdsList(ctx.subnetIds);
  resourcesVpcConfig.setVpcId(ctx.vpcId);

  return resourcesVpcConfig;
}

function toReplyCluster(
  ctx: AWS.EKS.Types.Cluster,
): CreateClusterReply.Cluster {
  const oidc = new CreateClusterReply.Cluster.Identity.Oidc();
  oidc.setIssuer(ctx.identity?.oidc?.issuer);

  const identity = new CreateClusterReply.Cluster.Identity();
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

  const cluster = new CreateClusterReply.Cluster();
  cluster.setArn(ctx.arn);
  cluster.setCertificateAuthority(
    toReplyClusterCertificate(ctx.certificateAuthority),
  );
  cluster.setClientRequestToken(ctx.clientRequestToken);
  cluster.setCreatedAt(ctx.createdAt.toUTCString());
  cluster.setEndpoint(ctx.endpoint);
  cluster.setIdentity(identity);
  cluster.setLogging(logging);
  cluster.setName(ctx.name);
  cluster.setPlatformVersion(ctx.platformVersion);
  cluster.setResourcesVpcConfig(
    toReplyClusterVpcConfigResponse(ctx.resourcesVpcConfig),
  );
  cluster.setRoleArn(ctx.roleArn);
  cluster.setStatus(ctx.status);
  cluster.setTagsList(tags);
  cluster.setVersion(ctx.version);

  return cluster;
}

function buildCreateClusterReply(
  response: AWS.EKS.Types.CreateClusterResponse,
): CreateClusterReply {
  const reply = new CreateClusterReply();
  reply.setCluster(toReplyCluster(response.cluster));

  return reply;
}

class AwsEks {
  constructor() {
    // These will be used as callbacks - so eventually, they are called in a way
    // that has them totally detached from their original context (in this case,
    // the instance of our class here.) This dirty mojo makes that not happen,
    // and ensures that the callbacks have reasonable encapsulation.
    this.createCluster = this.createCluster.bind(this);
  }

  addToServer(server: Server): void {
    logger.log("info", "Adding AWS EKS");
    server.addService(EKSService, {
      createCluster: this.createCluster,
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
    const request = buildCreateClusterRequest(call.request);
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

    const reply = buildCreateClusterReply(response);

    callback(null, reply);
  }
}

export const AwsEksService = new AwsEks();
