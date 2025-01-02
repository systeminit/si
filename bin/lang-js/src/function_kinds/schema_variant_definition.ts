import {
  failureExecution,
  Func,
  FunctionKind,
  ResultFailure,
  ResultSuccess,
  runCode,
} from "../function.ts";
import { RequestCtx } from "../request.ts";
import { Debug } from "../debug.ts";

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
  { executionId }: RequestCtx,
  _: SchemaVariantDefinitionFunc,
  code: string,
  timeout: number,
): Promise<SchemaVariantDefinitionResult> {
  let result: Record<string, unknown>;
  try {
    result = await runCode(
      code,
      FunctionKind.SchemaVariantDefinition,
      executionId,
      timeout,
      {},
    );
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
