import Debug from "debug";
import _ from "lodash";
import { VM, VMScript } from "vm2";
import { base64Decode } from "./base64";
import {
  failureExecution,
  FunctionKind,
  Request,
  ResultFailure,
  ResultSuccess,
} from "./function";
import { createSandbox } from "./sandbox";
import { createVm } from "./vm";
import { Component } from "./component";

const debug = Debug("langJs:codeGeneration");

export interface CodeGenerationRequest extends Request {
  handler: string;
  component: Component;
  codeBase64: string;
}

export type CodeGenerationResult =
  | CodeGenerationResultSuccess
  | CodeGenerationResultFailure;

export interface CodeGenerationResultSuccess extends ResultSuccess {
  data?: string;
}

export interface CodeGenerationResultFailure extends ResultFailure {
  something?: never;
}

export function executeCodeGeneration(request: CodeGenerationRequest): void {
  const code = base64Decode(request.codeBase64);

  debug({ code });

  // TODO: remove this, needed for now as it's messing with the generated yaml
  request.component.properties.maybe_sensitive_container_kind = undefined;

  const compiledCode = new VMScript(wrapCode(code, request.handler, request.component)).compile();
  debug({ code: compiledCode.code });
  const sandbox = createSandbox(FunctionKind.CodeGeneration, request.executionId);
  const vm = createVm(sandbox);

  const result = execute(vm, compiledCode, request.executionId);
  debug({ result });

  console.log(JSON.stringify(result));
}

function execute(
  vm: VM,
  code: VMScript,
  executionId: string
): CodeGenerationResult {
  let codeGenerationResult;
  try {
    codeGenerationResult = vm.run(code);
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
    data: codeGenerationResult,
    executionId,
  };
  return result;
}

function wrapCode(code: string, handle: string, component: Component): string {
  return code + `\n${handle}(${JSON.stringify(component)});\n`;
}
