import { Debug } from "../debug.ts";

import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
  runCode,
} from "../function.ts";
import { RequestCtx } from "../request.ts";
import { FunctionKind } from "../function.ts";

const debug = Debug("langJs:before");

export interface BeforeFunc extends Func {
  arg: unknown;
}

export type BeforeResultSuccess = ResultSuccess;

export type BeforeResultFailure = ResultFailure;

export type BeforeResult = BeforeResultSuccess | BeforeResultFailure;

async function execute(
  { executionId }: RequestCtx,
  { arg }: BeforeFunc,
  code: string,
): Promise<BeforeResult> {
  try {
    await runCode(
      code,
      FunctionKind.Before,
      executionId,
      arg as Record<string, unknown>,
    );
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }

  return {
    protocol: "result",
    status: "success",
    executionId,
  };
}

const wrapCode = (code: string) => `
async function run(arg) {
  ${code}
  const returnValue = await main(arg);
  return returnValue;
}`;

export default {
  debug,
  execute,
  wrapCode,
};
