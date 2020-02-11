import {
  EC2Client,
  CreateKeyPairCommand,
  CreateKeyPairCommandInput,
  CreateKeyPairCommandOutput,
} from "@aws-sdk/client-ec2";

import { logger } from "@/logger";
import { Server, ServerUnaryCall, sendUnaryData } from "grpc";
import { Ec2Service } from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.ec2_grpc_pb";
import {
  CreateKeyPairRequest,
  CreateKeyPairReply,
  Error as PError,
} from "@/generated/si-external-api-gateway/proto/si.external_api_gateway.aws.ec2_pb";

class AwsEc2 {
  constructor() {
    // These will be used as callbacks - so eventually, they are called in a way
    // that has them totally detached from their original context (in this case,
    // the instance of our class here.) This dirty mojo makes that not happen,
    // and ensures that the callbacks have reasonable encapsulation.
    this.createKeyPair = this.createKeyPair.bind(this);
  }

  addToServer(server: Server): void {
    logger.log("info", "Adding AWS EC2");
    server.addService(Ec2Service, {
      createKeyPair: this.createKeyPair,
    });
  }

  async createKeyPair(
    call: ServerUnaryCall<CreateKeyPairRequest>,
    callback: sendUnaryData<CreateKeyPairReply>,
  ): Promise<void> {
    const client = new EC2Client({ region: "us-west-1" });
    const commandInputs: CreateKeyPairCommandInput = {
      KeyName: call.request.getKeyname(),
      DryRun: call.request.getDryRun(),
    };
    const command = new CreateKeyPairCommand(commandInputs);
    const reply = new CreateKeyPairReply();
    try {
      const results: CreateKeyPairCommandOutput = await client.send(command);
      reply.setKeyFingerprint(results.KeyFingerprint);
      reply.setKeyPairId(results.KeyPairId);
      reply.setKeyMaterial(results.KeyMaterial);
      reply.setKeyName(results.KeyName);
    } catch (err) {
      const error = new PError();
      error.setCode(err.code);
      error.setMessage(err.message);
      reply.setError(error);
    }
    callback(null, reply);
  }
}

export const AwsEc2Service = new AwsEc2();
