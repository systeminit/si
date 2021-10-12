import { consoleObject as console } from "./sandbox/console";
import _ from "lodash";
import { RemoteFunctionRequest } from "./remote_function";

export type Sandbox = Record<string, any>;

export class UnknownSandboxKind extends Error {
  constructor(kind: string) {
    const message = `Unknown sandbox kind: ${kind}; bug!`;
    super(message);
    this.name = "UnknownSandboxKind";
  }
}

export const resolverSandbox = {
  console,
  _,
};

export function createSandbox(kind: RemoteFunctionRequest["kind"]): Sandbox {
  if (kind == "resolver") {
    return resolverSandbox;
  } else {
    throw new UnknownSandboxKind(kind);
  }
}
