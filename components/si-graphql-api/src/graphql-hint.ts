import * as fs from "fs";
import * as TOML from "@iarna/toml";
import { ServiceDescription } from "@/services";

export interface GraphqlHintMethod {
  query?: boolean;
  mutation?: boolean;
  skipauth?: boolean;
}

export interface GraphqlHintMessage {
  [fieldName: string]: {
    has_one?: {
      to: string;
      grpcServiceName: string;
      method: string;
      type: string;
    };
    skip?: boolean;
  };
}

export interface GraphqlHint {
  protoPackageName?: string;
  service?: {
    [serviceName: string]: {
      [methodName: string]: GraphqlHintMethod;
    };
  };
  message?: {
    [messageName: string]: GraphqlHintMessage;
  };
}

export class GraphqlHintLoader {
  hints: GraphqlHint[];

  constructor({ services }: { services: ServiceDescription[] }) {
    this.hints = [];
    for (const service of services) {
      const hintmap: GraphqlHint = TOML.parse(
        fs.readFileSync(service.graphqlHintPath(), { encoding: "utf8" }),
      );
      console.log("warn", "toml", { hintmap });
      this.hints.push(hintmap);
    }
  }
}
