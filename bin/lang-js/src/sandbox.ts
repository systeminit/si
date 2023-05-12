import os from "os";
import fs from "fs";
import path from "path";
import fetch from "node-fetch";

import _ from "lodash";
import yaml from "js-yaml";

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

function resolverFunctionSandbox(executionId: string): Sandbox {
  return {
    // Is there any risk leaking this function plainly here? It smells like a risk for RCE outside of the sandbox
    YAML: { stringify: yaml.dump },
    fetch,
    // definitely a risk
    // lol
    siExec: makeExec(executionId),
    os, // This certainly is bad
    fs, // This certainly is bad
    path, // This certainly is bad
  };
}

const workflowResolveSandbox = {};

const validationSandbox = {};

function reconciliationSandbox(executionId: string): Sandbox {
  return {
    siExec: makeExec(executionId),
  };
}

function commandRunSandbox(executionId: string): Sandbox {
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
        ...resolverFunctionSandbox(executionId),
      };
    case FunctionKind.WorkflowResolve:
      return {
        ...commonSandbox(executionId),
        ...workflowResolveSandbox,
      };
    case FunctionKind.CommandRun:
      return {
        ...commonSandbox(executionId),
        ...commandRunSandbox(executionId),
      };
    case FunctionKind.Validation:
      return {
        ...commonSandbox(executionId),
        ...validationSandbox,
      };
    case FunctionKind.Reconciliation:
      return {
        ...commonSandbox(executionId),
        ...reconciliationSandbox,
      };
    default:
      throw new UnknownSandboxKind(kind);
  }
}
