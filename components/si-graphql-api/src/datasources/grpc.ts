import { DataSource } from "apollo-datasource";

import { ServiceDescription } from "@/services";
import grpcCaller = require("grpc-caller");
import { logger } from "@/logger";

export class GrpcServiceBroker {
  services: {
    [key: string]: {
      client: grpcCaller;
    };
  };

  constructor({ services }: { services: ServiceDescription[] }) {
    this.services = {};

    for (const sd of services) {
      const caller = grpcCaller(
        sd.address,
        {
          file: sd.protobufPath(),
          load: {
            keepCase: false,
            longs: String,
            defaults: true,
            oneofs: true,
          },
        },
        sd.grpcServiceName,
      );

      logger.log("info", "Loading Service", { service: sd, caller });

      this.services[sd.grpcServiceName] = {
        client: caller,
      };
    }
  }
}

export class Grpc extends DataSource {
  broker: GrpcServiceBroker;
  config: any;

  constructor({ broker }: { broker: GrpcServiceBroker }) {
    super();
    this.broker = broker;
  }

  initialize(config): void {
    this.config = config;
  }

  service(service: string): any {
    logger.log("warn", "getting service", { service, broker: this.broker });
    return this.broker.services[service].client;
  }
}
