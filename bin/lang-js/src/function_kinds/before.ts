import Debug from "debug";
import {NodeVM} from "vm2";

import {
  executor,
  Func,
  FunctionKind,
  ResultFailure,
  ResultSuccess,
} from "../function";
import {RequestCtx} from "../request";

const debug = Debug("langJs:validation");

export type BeforeFunc = Func;

export interface BeforeResultSuccess extends ResultSuccess {
  definition: object;
}

export type BeforeResultFailure = ResultFailure;

export type BeforeResult =
  | BeforeResultSuccess
  | BeforeResultFailure;

export async function executeBefore(
  func: BeforeFunc,
  ctx: RequestCtx,
): Promise<void> {
  await executor(
    ctx, func,
    FunctionKind.Before,
    debug,
    wrapCode,
    execute
  );
}

// TODO Implement execute and wrap code for Before funcs
async function execute(
  vm: NodeVM,
  {executionId}: RequestCtx,
  _: BeforeFunc,
  _code: string,
): Promise<BeforeResult> {
  // let result: Record<string, unknown>;
  // try {
  //   const runner = vm.run(code);
  //   result = await new Promise((resolve) => {
  //     runner((resolution: Record<string, unknown>) => resolve(resolution));
  //   });
  //   debug({result: JSON.stringify(result)});
  // } catch (err) {
  //   return failureExecution(err as Error, executionId);
  // }

  return {
    protocol: "result",
    status: "success",
    executionId,
    definition: {},
  };
}

const wrapCode = (code: string, handler: string) => `
module.exports = function(callback) {
  ${code}
  const returnValue = ${handler}();
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
