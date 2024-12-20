import * as _ from "lodash-es";
import process from "node:process";
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
import { rawStorage, rawStorageRequest } from "./sandbox/requestStorage.ts";
import { Debug, Debugger } from "./debug.ts";
import { transpile } from "jsr:@deno/emit";
import * as w from "./worker.js";

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
    // Set process environment variables, set from requestStorage
    {
      const requestStorageEnv = rawStorageRequest().env();
      for (const key in requestStorageEnv) {
        process.env[key] = requestStorageEnv[key];
      }
    }
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

class TimeoutError extends Error {
  constructor(seconds: number) {
    super(`function timed out after ${seconds} seconds`);
    this.name = "TimeoutError";
  }
}

function timer(seconds: number): Promise<never> {
  const ms = seconds * 1000;
  return new Promise((_, reject) => {
    setTimeout(() => reject(new TimeoutError(seconds)), ms);
  });
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
    wrapCode: (code: string) => string;
    execute: (
      ctx: RequestCtx,
      func: F,
      code: string,
    ) => Promise<JoiValidationResult | Result>;
  },
) {
  let originalCode = "";
  if (!_.isEmpty(func.codeBase64)) {
    originalCode = base64Decode(func.codeBase64);
  }

  const code = wrapCode(originalCode);

  // Following section throws on timeout or execution error
  const result = await Promise.race([
    execute(ctx, func, code),
    timer(timeout),
  ]);
  debug({ result });
  return result;
}

export async function runCode(
  code: string,
  func_kind: FunctionKind,
  execution_id: string,
  with_arg?: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  const debug = Debug("runCode");
  code = await bundleCode(code);
  const currentEnv = rawStorage().env;

  const worker = new Worker(new URL("./worker.js", import.meta.url), {
    type: "module",
    deno: {
      permissions: {
        import: true,
        env: true,
        net: true,
        read: true,
        run: true,
        sys: true,
        write: true,
      },
    },
  });

  return new Promise((resolve, reject) => {
    worker.postMessage({
      bundledCode: code,
      func_kind,
      execution_id,
      with_arg,
      env: currentEnv ?? {}
    });

    worker.onmessage = (event) => {
      const { result, env } = event.data;
      if (env) {
        Object.assign(rawStorage().env, env);
      }
      resolve(result);
      worker.terminate();
    };

    worker.onerror = (error) => {
      reject(error);
      worker.terminate();
    };
  });
}

async function bundleCode(code: string): Promise<string> {
  const tempDir = await Deno.makeTempDir();
  const tempFile = `${tempDir}/script.ts`;
  await Deno.writeTextFile(tempFile, code);
  const fileUrl = new URL(tempFile, import.meta.url);

  try {
    const bundled = (await transpile(fileUrl)).get(fileUrl.href) as string;
    return bundled;
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
}
