import Debug from "debug";
import {NodeVM} from "vm2";

import {
  executor,
  failureExecution,
  Func,
  FunctionKind,
  ResultFailure,
  ResultSuccess,
} from "../function";
import {RequestCtx} from "../request";

const debug = Debug("langJs:validation");

export interface ValidationFunc extends Func {
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


export async function executeValidation(
  func: ValidationFunc,
  ctx: RequestCtx,
) {
  return await executor(ctx, func, FunctionKind.Validation, debug, wrapCode, execute);
}

async function execute(
  vm: NodeVM,
  {executionId}: RequestCtx,
  {value}: ValidationFunc,
  code: string,
): Promise<ValidationResult> {
  let result: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    result = await new Promise((resolve) => {
      runner(value, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });
    debug({result: JSON.stringify(result)});
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
