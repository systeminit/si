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
  timeout: number,
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
      timeout,
      parsedArgs,
    );

    if (
      result.err && typeof result.err === "object" && "name" in result.err &&
      "message" in result.err
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: {
            UserCodeException: result.err.name as string,
          },
          message: result.err.message as string,
        },
      };
    }
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
  try {
    definition = JSON.parse(validationFormat);
  } catch (e) {
    const error = new Error('Invalid JSON format');
    error.name = 'JoiValidationJsonParsingError';
    throw error;
  }

  let schema;
  try {
    schema = Joi.build(definition);
  } catch (e) {
    const error = new Error('validationFormat must be of type object');
    error.name = 'JoiValidationFormatError';
    throw error;
  }

  const { error } = schema.validate(value);
  return { err: error ? error.message : undefined };
}`;

export default {
  debug,
  execute,
  wrapCode,
};
