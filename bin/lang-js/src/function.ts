import _ from "lodash";
import {Debugger} from "debug";
import {NodeVM} from "vm2";
import {base64ToJs} from "./base64";
import {createNodeVm} from "./vm";
import {createSandbox} from "./sandbox";
import {ctxFromRequest, Request, RequestCtx} from "./request";
import validation, {ValidationFunc} from "./function_kinds/validation";
import reconciliation, {
  ReconciliationFunc
} from "./function_kinds/reconciliation";
import resolver_function, {
  ResolverFunc
} from "./function_kinds/resolver_function";
import schema_variant_definition, {
  SchemaVariantDefinitionFunc
} from "./function_kinds/schema_variant_definition";
import action_run, {ActionRunFunc} from "./function_kinds/action_run";
import before from "./function_kinds/before";

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

export async function executeFunction(kind: FunctionKind, request: Request) {
  // Run Before Functions
  const ctx = ctxFromRequest(request)

  for (const beforeFunction of request.before || []) {
    await executor(ctx, beforeFunction, FunctionKind.Before, before)
  }

  // TODO Create Func types instead of casting request objs
  let result;
  switch (kind) {
    case FunctionKind.ActionRun:
      result = await executor(ctx, request as ActionRunFunc, kind, action_run);
      break;
    case FunctionKind.Reconciliation:
      result = await executor(ctx, request as ReconciliationFunc, kind, reconciliation);
      break;
    case FunctionKind.ResolverFunction:
      result = await executor(ctx, request as ResolverFunc, kind, resolver_function);

      console.log(
        JSON.stringify({
          protocol: "output",
          executionId: ctx.executionId,
          stream: "output",
          level: "info",
          group: "log",
          message: `Output: ${JSON.stringify(result, null, 2)}`,
        })
      );
      break;
    case FunctionKind.Validation:
      result = await executor(ctx, request as ValidationFunc, kind, validation);
      break;
    case FunctionKind.SchemaVariantDefinition:
      result = await executor(ctx, request as SchemaVariantDefinitionFunc, kind, schema_variant_definition)
      break;
    default:
      throw Error(`Unknown Kind variant: ${kind}`);
  }

  console.log(JSON.stringify(result));
}


export async function executor<F extends Func, Result>(
  ctx: RequestCtx,
  func: F,
  kind: FunctionKind,
  {debug, wrapCode, execute}: {
    debug: Debugger,
    wrapCode: (code: string, handler: string) => string,
    execute: (vm: NodeVM, ctx: RequestCtx, func: F, code: string) => Promise<Result>,
  },
) {
  const originalCode = base64ToJs(func.codeBase64);

  const code = wrapCode(originalCode, func.handler);

  debug({code});

  const vm = createNodeVm(createSandbox(kind, ctx.executionId));

  const result = await execute(vm, ctx, func, code);
  debug({result});

  return result;
}
