import { DataSource } from "apollo-datasource";
import * as path from "path";

import { ServiceDescription } from "@/services";
import * as grpcCaller from "grpc-caller";

export class GrpcServiceBroker {
  services: {
    [key: string]: {
      client: any;
    };
  };

  constructor({ services }: { services: ServiceDescription[] }) {
    this.services = {};

    for (const sd of services) {
      if (sd.dataOnly == true) {
        continue;
      }
      const caller = grpcCaller(
        sd.address,
        {
          file: sd.protobufPath(),
          load: {
            keepCase: false,
            longs: String,
            defaults: true,
            oneofs: true,
            includeDirs: [path.join("..")],
          },
        },
        sd.grpcServiceName,
      );

      console.log("Setting up service for", { sd });
      this.services[sd.serviceName] = {
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

  initialize(config: any): void {
    this.config = config;
  }

  service(service: string): any {
    return this.broker.services[service].client;
  }
}
