import Debug from "debug";
import _ from "lodash";
import { VM } from "vm2";
import { base64Decode } from "./base64";
import {
  failureExecution,
  FunctionKind,
  RequestWithCode,
  ResultFailure,
  ResultSuccess,
} from "./function";
import { createSandbox } from "./sandbox";
import { createNodeVm } from "./vm";
import { Component } from "./component";

const debug = Debug("langJs:codeGeneration");

export interface CodeGenerationRequest extends RequestWithCode {
  component: Component;
}

export type CodeGenerationResult =
  | CodeGenerationResultSuccess
  | CodeGenerationResultFailure;

export interface CodeGenerationResultSuccess extends ResultSuccess {
  data: {
    format: string;
    code: string;
  };
}

export interface CodeGenerationResultFailure extends ResultFailure {
  something?: never;
}

export async function executeCodeGeneration(request: CodeGenerationRequest): Promise<void> {
  const code = base64Decode(request.codeBase64);

  debug({ code });

  const wrappedCode = wrapCode(code, request.handler);
  debug({ code: wrappedCode });
  const sandbox = createSandbox(
    FunctionKind.CodeGeneration,
    request.executionId
  );
  const vm = createNodeVm(sandbox);

  const result = await execute(vm, wrappedCode, request.component, request.executionId);

  console.log(JSON.stringify(result));
}

async function execute(
  vm: VM,
  code: string,
  component: Component,
  executionId: string
): Promise<CodeGenerationResult> {
  let codeGenerationResult: Record<string, unknown>;
  try {
    const codeGenRunner = vm.run(code);
    codeGenerationResult = await new Promise((resolve) => {
        codeGenRunner(
          component,
          (resolution: Record<string, unknown>) => resolve(resolution)
        )
    })
  } catch (err) {
    return failureExecution(err, executionId);
  }

  if (_.isUndefined(codeGenerationResult) || _.isNull(codeGenerationResult)) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "InvalidReturnType",
        message: "Return type must not be null or undefined",
      },
    };
  }

  if (!_.isString(codeGenerationResult["format"])) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "FormatFieldWrongType",
        message: "The format field type must be string",
      },
    };
  }

  if (!_.isString(codeGenerationResult["code"])) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "CodeFieldWrongType",
        message: "The code field type must be string",
      },
    };
  }

  // TODO(fnichol): minimum checking of other result fields here...

  const result: CodeGenerationResultSuccess = {
    protocol: "result",
    status: "success",
    data: {
      format: codeGenerationResult["format"],
      code: codeGenerationResult["code"]
    },
    executionId,
  };
  return result;
}

function wrapCode(code: string, handle: string): string {
  const wrapped = `module.exports = function(component, callback) {
    ${code}
    const returnValue = ${handle}(component);
    if (returnValue instanceof Promise) {
      returnValue.then((data) => callback(data))
    } else {
      callback(returnValue);
    }
  };`;
  return wrapped;
}
