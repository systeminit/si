import * as protobuf from "protobufjs";
import { ServiceDescription } from "@/services";

export class ProtobufLoader {
  root: protobuf.Root;

  constructor({
    protos,
    services,
  }: {
    protos: string[];
    services: ServiceDescription[];
  }) {
    const paths = protos.concat(services.map(sd => sd.protobufPath()));
    // This is for our own code
    this.root = protobuf.loadSync(paths);
  }
}
