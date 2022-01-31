import _ from "lodash";

import { FunctionKind } from "./function";
import { makeConsole } from "./sandbox/console";
import { makeExec } from "./sandbox/exec";

export type Sandbox = Record<string, unknown>;

export class UnknownSandboxKind extends Error {
  constructor(kind: string) {
    const message = `Unknown sandbox kind: ${kind}; bug!`;
    super(message);
    this.name = "UnknownSandboxKind";
  }
}

function commonSandbox(executionId: string): Sandbox {
  return {
    console: makeConsole(executionId),
    _,
  };
}

const resolverFunctionSandbox = {};

const resourceSyncSandbox = {};

function qualificationCheckSandbox(executionId: string): Sandbox {
  return {
    siExec: makeExec(executionId),
  };
}

export function createSandbox(
  kind: FunctionKind,
  executionId: string
): Sandbox {
  switch (kind) {
    case FunctionKind.ResolverFunction:
      return {
        ...commonSandbox(executionId),
        ...resolverFunctionSandbox,
      };
    case FunctionKind.QualificationCheck:
      return {
        ...commonSandbox(executionId),
        ...qualificationCheckSandbox(executionId),
      };
    case FunctionKind.ResourceSync:
      return {
        ...commonSandbox(executionId),
        ...resourceSyncSandbox,
      };
    default:
      throw new UnknownSandboxKind(kind);
  }
}
