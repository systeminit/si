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

const debug = Debug("langJs:resolverFunction");

export interface ParentComponent {
  name: string;
  // TODO(fnichol): Highly, highly, highly TBD!
  properties: Record<string, unknown>;
}

export interface Component {
  name: string;
  // TODO(fnichol): Highly, highly, highly TBD!
  properties: Record<string, unknown>;
  parents: Array<ParentComponent>,
}

export interface ResolverFunctionRequest extends Request {
  handler: string;
  // Should this be optional?
  component: Component;
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
    wrapCode(code, request.handler, request.component)
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
  component: Component,
): string {
  return code + `\n${handle}(${JSON.stringify(component)});\n`;
}
