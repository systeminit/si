import { NodeVM } from "vm2";

import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
} from "../function";
import { RequestCtx } from "../request";
import { Debug } from "../debug";

const debug = Debug("langJs:schemaVariantDefinition");

export type SchemaVariantDefinitionFunc = Func;

export interface SchemaVariantDefinitionResultSuccess extends ResultSuccess {
  definition: object;
}

export type SchemaVariantDefinitionResultFailure = ResultFailure;

export type SchemaVariantDefinitionResult =
  | SchemaVariantDefinitionResultSuccess
  | SchemaVariantDefinitionResultFailure;

async function execute(
  vm: NodeVM,
  { executionId }: RequestCtx,
  _: SchemaVariantDefinitionFunc,
  code: string,
): Promise<SchemaVariantDefinitionResult> {
  let result: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    result = await new Promise((resolve) => {
      runner((resolution: Record<string, unknown>) => resolve(resolution));
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

export default {
  debug,
  execute,
  wrapCode,
};
