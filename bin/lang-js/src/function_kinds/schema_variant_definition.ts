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
import {RequestCtx} from "../index";
import {BeforeFunc} from "./before";

const debug = Debug("langJs:validation");

export type SchemaVariantDefinitionFunc = Func;

export interface SchemaVariantDefinitionResultSuccess extends ResultSuccess {
  definition: object;
}

export type SchemaVariantDefinitionResultFailure = ResultFailure;

export type SchemaVariantDefinitionResult =
  | SchemaVariantDefinitionResultSuccess
  | SchemaVariantDefinitionResultFailure;

export async function executeSchemaVariantDefinition(
  func: SchemaVariantDefinitionFunc,
  ctx: RequestCtx,
): Promise<void> {
  await executor(
    ctx, func,
    FunctionKind.SchemaVariantDefinition,
    debug,
    wrapCode,
    execute
  );
}

async function execute(
  vm: NodeVM,
  {executionId}: RequestCtx,
  _: BeforeFunc,
  code: string,
): Promise<SchemaVariantDefinitionResult> {
  let result: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    result = await new Promise((resolve) => {
      runner((resolution: Record<string, unknown>) => resolve(resolution));
    });
    debug({result: JSON.stringify(result)});
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
