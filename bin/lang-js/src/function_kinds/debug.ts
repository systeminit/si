import { Debug } from "../debug.ts";
import {
  failureExecution,
  Func,
  FunctionKind,
  ResultFailure,
  ResultSuccess,
} from "../function.ts";
import { runCode } from "../execution.ts";
import { RequestCtx } from "../request.ts";
import { Component } from "../component.ts";

const debug = Debug("langJs:debug");

export interface DebugFunc extends Func {
  component: Component;
  debugInput: unknown;
}

export type DebugFuncResult = DebugFuncResultSuccess | DebugFuncResultFailure;

export interface DebugFuncResultSuccess extends ResultSuccess {
  output: unknown;
}

export interface DebugFuncResultFailure extends ResultFailure {}

async function execute(
  { executionId }: RequestCtx,
  { component, debugInput, handler }: DebugFunc,
  code: string,
  timeout: number,
): Promise<DebugFuncResult> {
  let debugResult: unknown;
  try {
    debugResult = await runCode(
      code,
      handler,
      FunctionKind.Debug,
      executionId,
      timeout,
      {
        component,
        debugInput,
      },
    );
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }

  return {
    protocol: "result",
    status: "success",
    executionId,
    output: debugResult,
  };
}

const wrapCode = (code: string, handler: string) => `
  ${code}
  export { ${handler} };
`;

export default {
  debug,
  execute,
  wrapCode,
};
