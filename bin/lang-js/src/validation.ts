import Debug, { Debugger } from "debug";
import { base64ToJs } from "./base64";
import { NodeVM } from "vm2";

import {
  failureExecution,
  FunctionKind,
  RequestWithCode,
  ResultFailure,
  ResultSuccess,
} from "./function";

import { createSandbox } from "./sandbox";
import { createNodeVm } from "./vm";

const debug = Debug("langJs:validation");

export interface ValidationRequest extends RequestWithCode {
  value: unknown;
}

// TODO: validation funcs should return array of error messages since more than one thing can be
// wrong with a value
export interface ValidationResultSuccess extends ResultSuccess {
  valid: boolean;
  message?: string;
  //    link?: string;
  //    level?: string;
}

export interface ValidationResultFailure extends ResultFailure {
  valid?: never;
  message?: never;
  //    link?: never,
  //    level?: never,
}

export type ValidationResult =
  | ValidationResultSuccess
  | ValidationResultFailure;

// Could this function be used generically for all the request types?
// Seems like we could also maybe reduce wrapCode duplication a lot
export async function executor<Req extends RequestWithCode, Result>(
  request: Req,
  kind: FunctionKind,
  debug: Debugger,
  wrapCode: (code: string, handler: string) => string,
  execute: (vm: NodeVM, code: string, request: Req) => Promise<Result>
) {
  const originalCode = base64ToJs(request.codeBase64);

  const code = wrapCode(originalCode, request.handler);
  debug({ code });

  const vm = createNodeVm(createSandbox(kind, request.executionId));

  const result = await execute(vm, code, request);
  debug({ result });

  console.log(JSON.stringify(result));
}

export async function executeValidation(
  request: ValidationRequest
): Promise<void> {
  await executor(request, FunctionKind.Validation, debug, wrapCode, execute);
}

async function execute(
  vm: NodeVM,
  code: string,
  request: ValidationRequest
): Promise<ValidationResult> {
  const { executionId, value } = request;
  let result: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    result = await new Promise((resolve) => {
      runner(value, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });
    debug({ result: JSON.stringify(result) });
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }

  const invalidReturnType = "InvalidReturnType";
  if (typeof result["valid"] !== "boolean") {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: invalidReturnType,
        message: "field 'valid' must be boolean",
      },
    };
  }

  if (
    typeof result["message"] !== "string" &&
    typeof result["message"] !== "undefined"
  ) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: invalidReturnType,
        message: "field 'message' must be a string or undefined",
      },
    };
  }

  return {
    protocol: "result",
    status: "success",
    executionId,
    valid: result["valid"],
    message: result["message"],
  };
}

const wrapCode = (code: string, handler: string) => `
module.exports = function(value, callback) {
  ${code}
  const returnValue = ${handler}(value);
  if (returnValue instanceof Promise) {
    returnValue.then((data) => callback(data))
      .catch((err) => {
        callback({
          success: false,
          message: err.message,
        })
      });
  } else {
    callback(returnValue);
  }
};`;
