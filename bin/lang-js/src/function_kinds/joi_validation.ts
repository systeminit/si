import { Debug } from "../debug.ts";

import {
  failureExecution,
  Func,
  FunctionKind,
  ResultFailure,
  ResultSuccess,
  runCode,
} from "../function.ts";
import { RequestCtx } from "../request.ts";

const debug = Debug("langJs:joi_validation");

export interface JoiValidationFunc extends Func {
  value: unknown;
  validationFormat: string;
}

export type JoiValidationResultSuccess = ResultSuccess;

export type JoiValidationResultFailure = ResultFailure;

export type JoiValidationResult =
  | JoiValidationResultSuccess
  | JoiValidationResultFailure;

export interface JoiExecutionResult {
  err?: string;
}

async function execute(
  { executionId }: RequestCtx,
  args: JoiValidationFunc,
  code: string,
): Promise<JoiValidationResult> {
  try {
    // NOTE(victor): Joi treats null as a value, so even if .required()
    // isn't set it fails validations for typed props
    const parsedArgs = {
      ...args,
      value: args.value === null ? undefined : args.value,
    };

    const result = await runCode(
      code,
      FunctionKind.Validation,
      executionId,
      parsedArgs,
    );
    debug({ result });
    return {
      protocol: "result",
      status: "success",
      executionId,
      error: result.err as string,
    };
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
}

const wrapCode = (_: string) => `
async function run({ value, validationFormat }) {
  let definition;
  let message;
  try {
    definition = JSON.parse(validationFormat);
  } catch (e) {
    e.name = "JoiValidationJsonParsingError";
    message = e;
  }

  let schema;
  try {
    schema = Joi.build(definition);
  } catch (e) {
    e.name = "JoiValidationFormatError";
    e.message = e.message.replace("\\"value\\"", "validationFormat");
    message = e;
  }

  const { error } = schema.validate(value);
  return { "err": error };
}`;

export default {
  debug,
  execute,
  wrapCode,
};
