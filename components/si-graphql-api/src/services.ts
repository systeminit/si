import * as path from "path";

import { environment } from "@/environment";

export class ServiceDescription {
  serviceName: string;
  protoPackageName: string;
  graphqlTypePrefix: string;
  grpcServiceName: string;
  address: string;

  constructor({
    serviceName,
    protoPackageName,
    grpcServiceName,
    graphqlTypePrefix,
    address,
  }: {
    serviceName: string;
    protoPackageName: string;
    grpcServiceName: string;
    graphqlTypePrefix: string;
    address: string;
  }) {
    this.serviceName = serviceName;
    this.protoPackageName = protoPackageName;
    this.grpcServiceName = grpcServiceName;
    this.graphqlTypePrefix = graphqlTypePrefix;
    this.address = address;
  }

  protobufPath(): string {
    // Services are always peers here, for now.
    const dir = path.join(__dirname, "..", "..");
    return path.join(
      dir,
      this.serviceName,
      "proto",
      `${this.protoPackageName}.proto`,
    );
  }

  graphqlHintPath(): string {
    // Services are always peers here, for now.
    const dir = path.join(__dirname, "..", "..");
    return path.join(
      dir,
      this.serviceName,
      "proto",
      `${this.protoPackageName}.graphql.toml`,
    );
  }
}

// Add new GRPC services here, and they will get turned in to
// GraphQL endpoints automatically for you.
export const services = [
  new ServiceDescription({
    serviceName: "si-account",
    protoPackageName: "si.account",
    grpcServiceName: "Account",
    graphqlTypePrefix: "",
    address: environment.services["si-account"],
  }),
  // new ServiceDescription({
  //   serviceName: "si-ssh-key",
  //   protoPackageName: "ssh_key",
  //   graphqlTypePrefix: "SshKey",
  //   address: environment.services["si-ssh-key"],
  // }),
];
