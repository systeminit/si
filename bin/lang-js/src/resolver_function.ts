import Debug from "debug";
import _ from "lodash";
import { VM, VMScript } from "vm2";
import { base64Decode } from "./base64";
import {
  failureExecution,
  FunctionKind,
  Parameters,
  Request,
  ResultFailure,
  ResultSuccess,
} from "./function";
import { createSandbox } from "./sandbox";
import { createVm } from "./vm";

const debug = Debug("langJs:resolverFunction");

export interface ResolverFunctionRequest extends Request {
  handler: string;
  parameters?: Parameters;
  codeBase64: string;
}

export type ResolverFunctionResult =
  | ResolverFunctionResultSuccess
  | ResolverFunctionResultFailure;

export interface ResolverFunctionResultSuccess extends ResultSuccess {
  data: unknown;
  unset: boolean;
}

export interface ResolverFunctionResultFailure extends ResultFailure {
  data?: never;
  unset?: never;
}

export function executeResolverFunction(
  request: ResolverFunctionRequest
): void {
  const code = base64Decode(request.codeBase64);
  const compiledCode = new VMScript(
    wrapCode(code, request.handler, request.parameters)
  ).compile();
  debug({ code: compiledCode.code });
  const sandbox = createSandbox(
    FunctionKind.ResolverFunction,
    request.executionId
  );
  const vm = createVm(sandbox);

  const result = execute(vm, compiledCode, request.executionId);
  debug({ result });

  console.log(JSON.stringify(result));
}

function execute(
  vm: VM,
  code: VMScript,
  executionId: string
): ResolverFunctionResult {
  let resolverFunctionResult;
  try {
    resolverFunctionResult = vm.run(code);
  } catch (err) {
    return failureExecution(err, executionId);
  }

  if (_.isUndefined(resolverFunctionResult)) {
    vm.sandbox.console.debug("function returned undefined");
    return {
      protocol: "result",
      status: "success",
      executionId,
      data: resolverFunctionResult,
      unset: true,
    };
  } else if (
    _.isString(resolverFunctionResult) ||
    _.isNumber(resolverFunctionResult) ||
    _.isBoolean(resolverFunctionResult) ||
    _.isPlainObject(resolverFunctionResult) ||
    _.isArray(resolverFunctionResult) ||
    _.isNull(resolverFunctionResult)
  ) {
    return {
      protocol: "result",
      status: "success",
      executionId,
      data: resolverFunctionResult,
      unset: false,
    };
  } else {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "InvalidReturnType",
        message:
          "Return type must be one of: " +
          "[string, number, boolean, object, array, null]",
      },
    };
  }
}

function wrapCode(
  code: string,
  handle: string,
  parameters?: Parameters
): string {
  if (!parameters) {
    parameters = {};
  }
  return code + `\n${handle}(${JSON.stringify(parameters)});\n`;
}
