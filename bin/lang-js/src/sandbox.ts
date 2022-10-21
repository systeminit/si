import os from "os";
import fs from "fs";
import path from "path";
import fetch from "node-fetch";

import _ from "lodash";
import yaml from "js-yaml";

import {FunctionKind} from "./function";
import {makeConsole} from "./sandbox/console";
import {makeExec} from "./sandbox/exec";

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

function codeGenerationSandbox(executionId: string) {
  return {
    // Is there any risk leaking this function plainly here? It smells like a risk for RCE outside of the sandbox
    YAML: {stringify: yaml.dump},
    // definitely a risk
    siExec: makeExec(executionId),
  };
};

const confirmationSandbox = {};

const workflowResolveSandbox = {};

const validationSandbox = {};

function commandRunSandbox(executionId: string): Sandbox {
  return {
    siExec: makeExec(executionId),
  };
}

function qualificationCheckSandbox(executionId: string): Sandbox {
  return {
    siExec: makeExec(executionId),
    fetch,
    os, // This certainly is bad
    fs, // This certainly is bad
    path, // This certainly is bad
  };
}

export function createSandbox(
  kind: FunctionKind,
  executionId: string
): Sandbox {
  switch (kind) {
    case FunctionKind.CodeGeneration:
      return {
        ...commonSandbox(executionId),
        ...codeGenerationSandbox(executionId),
      };
    case FunctionKind.QualificationCheck:
      return {
        ...commonSandbox(executionId),
        ...qualificationCheckSandbox(executionId),
      };
    case FunctionKind.ResolverFunction:
      return {
        ...commonSandbox(executionId),
        ...resolverFunctionSandbox,
      };
    case FunctionKind.Confirmation:
      return {
        ...commonSandbox(executionId),
        ...confirmationSandbox,
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
      }
    default:
      throw new UnknownSandboxKind(kind);
  }
}
