import * as path from "path";

import { environment } from "@/environment";

export class ServiceDescription {
  serviceName: string;
  protoPackageName: string;
  graphqlTypePrefix: string;
  grpcServiceName: string;
  address: string;
  dataOnly: boolean;

  constructor({
    serviceName,
    protoPackageName,
    grpcServiceName,
    graphqlTypePrefix,
    address,
    dataOnly,
  }: {
    serviceName: string;
    protoPackageName: string;
    grpcServiceName: string;
    graphqlTypePrefix: string;
    address: string;
    dataOnly?: boolean;
  }) {
    this.serviceName = serviceName;
    this.protoPackageName = protoPackageName;
    this.grpcServiceName = grpcServiceName;
    this.graphqlTypePrefix = graphqlTypePrefix;
    this.address = address;
    this.dataOnly = dataOnly;
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
  new ServiceDescription({
    serviceName: "si-data",
    protoPackageName: "si.data",
    grpcServiceName: "Data",
    graphqlTypePrefix: "Data",
    address: environment.services["si-account"],
    dataOnly: true,
  }),
  new ServiceDescription({
    serviceName: "si-ssh-key",
    protoPackageName: "si.ssh_key",
    grpcServiceName: "SshKey",
    graphqlTypePrefix: "SshKey",
    address: environment.services["si-ssh-key"],
  }),
  new ServiceDescription({
    serviceName: "si-aws-eks-cluster-runtime",
    protoPackageName: "si.aws_eks_cluster_runtime",
    grpcServiceName: "AwsEksClusterRuntime",
    graphqlTypePrefix: "AwsEksClusterRuntime",
    address: environment.services["si-aws-eks-cluster-runtime"],
  }),

  // new ServiceDescription({
  //   serviceName: "si-ssh-key",
  //   protoPackageName: "ssh_key",
  //   graphqlTypePrefix: "SshKey",
  //   address: environment.services["si-ssh-key"],
  // }),
];
