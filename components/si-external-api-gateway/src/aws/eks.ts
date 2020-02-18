import {
  EKSClient,
  CreateClusterCommand,
  CreateClusterInput,
  CreateClusterOutput,
} from "@aws-sdk/client-eks";

import { logger } from "@/logger";
import { Server, ServerUnaryCall, sendUnaryData } from "grpc";
import { EKSService } from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.eks_grpc_pb";
import {
  CreateClusterRequest,
  CreateClusterReply,
  Error as PError,
} from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.eks_pb";

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
    const client = new EKSClient({ region: "us-east-2" });
    const resourcesVpcConfig = call.request.getResourcesVpcConfig();

    const commandInputs: CreateClusterInput = {
      name: call.request.getName(),
      version: call.request.getVersion(),
      roleArn: call.request.getRoleArn(),
      logging: call.request.getLogging(),
      clientRequestToken: call.request.getClientRequestToken(),
      resourcesVpcConfig: {
        subnetIds: resourcesVpcConfig.getSubnetIdsList(),
        securityGroupIds: resourcesVpcConfig.getSecurityGroupIdsList(),
        endpointPublicAccess: resourcesVpcConfig.getEndpointPublicAccess(),
        endpointPrivateAccess: resourcesVpcConfig.getEndpointPrivateAccess(),
      },
    };
    const command = new CreateClusterCommand(commandInputs);
    const reply = new CreateClusterReply();
    try {
      const results: CreateClusterOutput = await client.send(command);
      const cluster = reply.getCluster();
      cluster.setCreatedAt(results.createdAt);
      cluster.setResourcesVpcConfig(results.resourcesVpcConfig);
      cluster.setLogging(results.logging);
      cluster.setCertificateAuthority(results.certificateAuthority);
      reply.setCluster(cluster);
    } catch (err) {
      const error = new PError();
      error.setCode(err.code);
      error.setMessage(err.message);
      reply.setError(error);
    }
    callback(null, reply);
  }
}

export const AwsEksService = new AwsEks();
