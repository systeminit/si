import * as _ from "lodash-es";
import { NodeVM } from "vm2";
import { Debug } from "../debug";
import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
} from "../function";
import { Component } from "../component";
import { RequestCtx } from "../request";

const debug = Debug("langJs:management");

export interface ManagementFunc extends Func {
  thisComponent: Component
}

export type ManagementFuncResult =
    | ManagementFuncResultSuccess
    | ManagementFuncResultFailure;

export interface ManagementOperations {
  update: { [key: string]: {
    properties?: object;
  } }
}

export interface ManagementFuncResultSuccess extends ResultSuccess {
  operations?: ManagementOperations,
  message?: string;
}
export interface ManagementFuncResultFailure extends ResultFailure { }

async function execute(
  vm: NodeVM,
  { executionId }: RequestCtx,
  { thisComponent }: ManagementFunc,
  code: string,
): Promise<ManagementFuncResult> {
  let managementResult: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    managementResult = await new Promise((resolve) => {
      runner(thisComponent.properties, (resolution: Record<string, unknown>) => resolve(resolution));
    });
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
  return {
    protocol: "result",
    status: "success",
    executionId,
    operations: managementResult.ops as ManagementOperations | undefined,
    message: managementResult.message as string | undefined,
  };
}

// Should we wrap this in a try/catch ?
const wrapCode = (code: string, handle: string) => `
module.exports = function(thisComponent, callback) {
  ${code}
  const returnValue = ${handle}(thisComponent);
  if (returnValue instanceof Promise) {
    returnValue.then((data) => callback(data))
  } else {
    callback(returnValue);
  }
};`;

export default {
  debug,
  execute,
  wrapCode,
};
