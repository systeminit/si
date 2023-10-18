import _ from "lodash";
import {Debugger} from "debug";
import {NodeVM} from "vm2";
import {base64ToJs} from "./base64";
import {createNodeVm} from "./vm";
import {createSandbox} from "./sandbox";
import {RequestCtx} from "./index";

export enum FunctionKind {
  ActionRun = "actionRun",
  Before = "before",
  ResolverFunction = "resolverfunction",
  Validation = "validation",
  Reconciliation = "reconciliation",
  SchemaVariantDefinition = "schemaVariantDefinition",
}

export function functionKinds(): Array<string> {
  return _.values(FunctionKind)
}

export type Parameters = Record<string, unknown>;

export interface Func {
  handler: string;
  codeBase64: string;
}

export interface Result {
  protocol: "result";
}

export interface ResultSuccess extends Result {
  status: "success";
  executionId: string;
  error?: string;
}

export interface ResultFailure extends Result {
  status: "failure";
  executionId: string;
  error: {
    kind: string;
    message: string;
  };
}

export function failureExecution(
  err: Error,
  executionId: string
): ResultFailure {
  // `executionId` may not have been determined if the request JSON fails to
  // parse, message is malformed, etc. In this case an empty string can signal
  // that an id could not be determined at this point
  if (!executionId) {
    executionId = "";
  }
  return {
    protocol: "result",
    status: "failure",
    executionId,
    error: {
      kind: err.name,
      message: err.message,
    },
  };
}

export interface OutputLine {
  protocol: "output";
  executionId: string;
  stream: "stdout" | "stderr";
  level: "debug" | "info" | "warn" | "error";
  group?: string;
  message: string;
}

export async function executor<F extends Func, Result>(
  ctx: RequestCtx,
  func: F,
  kind: FunctionKind,
  debug: Debugger,
  wrapCode: (code: string, handler: string) => string,
  execute: (vm: NodeVM, ctx: RequestCtx, func: F, code: string) => Promise<Result>,
  afterExecute?: (result: Result) => void,
) {
  const originalCode = base64ToJs(func.codeBase64);

  const code = wrapCode(originalCode, func.handler);
  debug({code});

  const vm = createNodeVm(createSandbox(kind, ctx.executionId));

  const result = await execute(vm, ctx, func, code);
  debug({result});

  if (afterExecute) {
    afterExecute(result);
  }

  console.log(JSON.stringify(result));
}
