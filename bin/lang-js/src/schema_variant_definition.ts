import Debug, { Debugger } from "debug";
import { NodeVM } from "vm2";
import { base64ToJs } from "./base64";

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

export type SchemaVariantDefinitionRequest = RequestWithCode;

export interface SchemaVariantDefinitionResultSuccess extends ResultSuccess {
  definition: object;
}

export type SchemaVariantDefinitionResultFailure = ResultFailure;

export type SchemaVariantDefinitionResult =
  | SchemaVariantDefinitionResultSuccess
  | SchemaVariantDefinitionResultFailure;

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

export async function executeSchemaVariantDefinition(
  request: SchemaVariantDefinitionRequest
): Promise<void> {
  await executor(request, FunctionKind.SchemaVariantDefinition, debug, wrapCode, execute);
}

async function execute(
  vm: NodeVM,
  code: string,
  request: SchemaVariantDefinitionRequest
): Promise<SchemaVariantDefinitionResult> {
  const { executionId } = request;
  let result: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    result = await new Promise((resolve) => {
      runner((resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });
    debug({ result: JSON.stringify(result) });
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }

  return {
    protocol: "result",
    status: "success",
    executionId,
    definition: result as object,
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
