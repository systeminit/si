import { NodeVM } from "vm2";
import { Debug } from "../debug";

import {
  failureExecution,
  Func, ResultFailure, ResultSuccess,
} from "../function";
import { RequestCtx } from "../request";

const debug = Debug("langJs:joi_validation");

export interface JoiValidationFunc extends Func {
  value: unknown;
  validationFormat: string;
}

export type JoiValidationResultSuccess = ResultSuccess;

export type JoiValidationResultFailure = ResultFailure;

export type JoiValidationResult =
  JoiValidationResultSuccess
  | JoiValidationResultFailure;

export interface JoiExecutionResult {
  err?: string;
}

async function execute(
  vm: NodeVM,
  { executionId }: RequestCtx,
  args: JoiValidationFunc,
  code: string,
): Promise<JoiValidationResult> {
  try {
    const runner = vm.run(code);

    // NOTE(victor): Joi treats null as a value, so even if .required()
    // isn't set it fails validations for typed props
    const parsedArgs = {
      ...args,
      value: args.value === null ? undefined : args.value,
    };

    const resolution: JoiExecutionResult = await new Promise((resolve) => {
      runner(parsedArgs, (resolution: JoiExecutionResult) => resolve(resolution));
    });
    debug({ result: resolution });

    return {
      protocol: "result",
      status: "success",
      executionId,
      error: resolution.err,
    };
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
}

const wrapCode = (_: string, __: string) => `
module.exports = function({ value, validationFormat }, callback) {
  let schema;
  try {
    const definition = JSON.parse(validationFormat);
    schema = Joi.build(definition);
  } catch (e) {
    e.name = "JoiValidationFormatError";
    throw e;
  }

  const { error } = schema.validate(value);
  const err = error?.message;
  callback({ err });
};`;

export default {
  debug,
  execute,
  wrapCode,
};
