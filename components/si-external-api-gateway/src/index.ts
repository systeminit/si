import grpc from "grpc";

import { environment } from "@/environment";
import { logger } from "@/logger";
import { AwsEc2Service } from "@/aws/ec2";

logger.log("info", "*** Starting the external api gateway ***");
logger.log("info", "Loading services");
const server = new grpc.Server();
AwsEc2Service.addToServer(server);

const bindTo = `0.0.0.0:${environment.port}`;

logger.log("info", `Starting server on: ${bindTo}`);
server.bind(bindTo, grpc.ServerCredentials.createInsecure());
server.start();
