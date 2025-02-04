import * as _ from "npm:lodash-es";
import { base64Decode } from "./base64.ts";
import { ctxFromRequest, Request, RequestCtx } from "./request.ts";
import joi_validation, {
  JoiValidationFunc,
  JoiValidationResult,
} from "./function_kinds/joi_validation.ts";
import resolver_function, {
  ResolverFunc,
} from "./function_kinds/resolver_function.ts";
import schema_variant_definition, {
  SchemaVariantDefinitionFunc,
} from "./function_kinds/schema_variant_definition.ts";
import management_run, { ManagementFunc } from "./function_kinds/management.ts";
import action_run, { ActionRunFunc } from "./function_kinds/action_run.ts";
import before from "./function_kinds/before.ts";
import { rawStorage } from "./sandbox/requestStorage.ts";
import { Debugger } from "./debug.ts";
import { Debug } from "./debug.ts";
import { bundleCode } from "./transpile.ts";
import { createSandbox } from "./sandbox.ts";

const debug = Debug("langJs:function");

export enum FunctionKind {
  ActionRun = "actionRun",
  Before = "before",
  Management = "management",
  ResolverFunction = "resolverfunction",
  Validation = "validation",
  SchemaVariantDefinition = "schemaVariantDefinition",
}

export function functionKinds(): Array<string> {
  return _.values(FunctionKind);
}

export type Parameters = Record<string, unknown>;

export interface Func {
  codeBase64: string;
  handler: string;
}

class TimeoutError extends Error {
  constructor(seconds: number) {
    super(`function timed out after ${seconds} seconds`);
    this.name = "TimeoutError";
  }
}

export interface Result {
  protocol: "result";
}

export interface ResultSuccess extends Result {
  status: "success";
  executionId: string;
  error?: string;
}

// Js sandbox errors that don't get categorized return as a generic UserCodeException to the rust enum
export interface ResultFailure extends Result {
  status: "failure";
  executionId: string;
  error: {
    kind: string | {
      "UserCodeException": string;
    };
    message: string;
  };
}

export function failureExecution(
  err: Error,
  executionId: string,
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
      kind: { UserCodeException: err.name },
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

export async function executeFunction(
  kind: FunctionKind,
  request: Request,
  timeout: number,
) {
  // Run Before Functions
  const ctx = ctxFromRequest(request);

  for (const beforeFunction of request.before || []) {
    await executor(ctx, beforeFunction, timeout, before);
  }

  // TODO Create Func types instead of casting request objs
  let result;
  switch (kind) {
    case FunctionKind.ActionRun:
      result = await executor(
        ctx,
        request as ActionRunFunc,
        timeout,
        action_run,
      );

      console.log(
        JSON.stringify({
          protocol: "output",
          executionId: ctx.executionId,
          stream: "output",
          level: "info",
          group: "log",
          message: `Output: ${JSON.stringify(result, null, 2)}`,
        }),
      );

      break;
    case FunctionKind.ResolverFunction:
      result = await executor(
        ctx,
        request as ResolverFunc,
        timeout,
        resolver_function,
      );

      console.log(
        JSON.stringify({
          protocol: "output",
          executionId: ctx.executionId,
          stream: "output",
          level: "info",
          group: "log",
          message: `Output: ${JSON.stringify(result, null, 2)}`,
        }),
      );
      break;
    case FunctionKind.Validation:
      result = await executor(
        ctx,
        request as JoiValidationFunc,
        timeout,
        joi_validation,
      );
      break;
    case FunctionKind.SchemaVariantDefinition:
      result = await executor(
        ctx,
        request as SchemaVariantDefinitionFunc,
        timeout,
        schema_variant_definition,
      );
      break;
    case FunctionKind.Management:
      result = await executor(
        ctx,
        request as ManagementFunc,
        timeout,
        management_run,
      );
      break;
    default:
      throw Error(`Unknown Kind variant: ${kind}`);
  }

  console.log(JSON.stringify(result));
}

export async function executor<F extends Func, Result>(
  ctx: RequestCtx,
  func: F,
  timeout: number,
  {
    debug,
    execute,
    wrapCode,
  }: {
    debug: Debugger;
    wrapCode: (code: string, handler: string) => string;
    execute: (
      ctx: RequestCtx,
      func: F,
      code: string,
      timeout: number,
    ) => Promise<JoiValidationResult | Result>;
  },
) {
  let originalCode = "";
  if (!_.isEmpty(func.codeBase64)) {
    originalCode = base64Decode(func.codeBase64);
  }

  const code = wrapCode(originalCode, func.handler);

  // Following section throws on timeout or execution error
  const result = await execute(ctx, func, code, timeout);
  debug({ result });
  return result;
}

export async function runCode(
  code: string,
  func_kind: FunctionKind,
  execution_id: string,
  timeout: number,
  with_arg?: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  code = await bundleCode(code);

  const sandbox = createSandbox(func_kind, execution_id);
  const keys = Object.keys(sandbox);
  const values = Object.values(sandbox);

  for (const [key, value] of Object.entries(rawStorage().env || {})) {
    Deno.env.set(key, value);
  }

  const func = new Function(
    ...keys,
    "with_arg",
    `
      return (async () => {
        ${code}
        try {
          return await run(with_arg);
        } catch (e) {
          throw e
        }
       })()
    `,
  );

  const timeoutId = setTimeout(() => {
    throw new TimeoutError(timeout);
  }, timeout * 1000);

  try {
    const result = await func(...values, with_arg);
    debug({ result });
    clearTimeout(timeoutId);
    return result;
  } catch (e: unknown) {
    debug({ error: e });
    clearTimeout(timeoutId);

    throw e instanceof Error ? e : new Error(String(e));
  }
}
