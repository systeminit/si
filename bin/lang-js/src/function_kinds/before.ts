import Debug from "debug";
import {NodeVM} from "vm2";

import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
} from "../function";
import {RequestCtx} from "../request";

const debug = Debug("langJs:validation");

export type BeforeFunc = Func;

export type BeforeResultSuccess = ResultSuccess;

export type BeforeResultFailure = ResultFailure;

export type BeforeResult =
  | BeforeResultSuccess
  | BeforeResultFailure;

// TODO Implement execute and wrap code for Before funcs
async function execute(
  vm: NodeVM,
  {executionId}: RequestCtx,
  _: BeforeFunc,
  code: string,
): Promise<BeforeResult> {
  try {
    const runner = vm.run(code);
    await new Promise((resolve) => {
      runner((resolution: Record<string, unknown>) => resolve(resolution));
    });
    debug({result: "<void>"});

  } catch (err) {
    return failureExecution(err as Error, executionId);
  }

  return {
    protocol: "result",
    status: "success",
    executionId,
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

export default {
  debug,
  execute,
  wrapCode
}
