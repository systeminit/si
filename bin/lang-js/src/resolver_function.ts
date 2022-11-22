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

const debug = Debug("langJs:resolverFunction");

export interface ResolverComponent {
  data: Component;
  parents: Array<Component>;
}

export interface ResolverFunctionRequest extends RequestWithCode {
  // Should this be optional?
  component: ResolverComponent;
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

export async function executeResolverFunction(
  request: ResolverFunctionRequest
): Promise<void> {
  let code = base64Decode(request.codeBase64);
  debug({ code });

  code = wrapCode(code, request.handler);
  debug({ code });

  const sandbox = createSandbox(
    FunctionKind.ResolverFunction,
    request.executionId
  );
  const vm = createNodeVm(sandbox);

  const result = await execute(
    vm,
    code,
    request.component,
    request.executionId
  );
  debug({ result });

  console.log(JSON.stringify(result));
}

async function execute(
  vm: VM,
  code: string,
  component: ResolverComponent,
  executionId: string
): Promise<ResolverFunctionResult> {
  let resolverFunctionResult: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    resolverFunctionResult = await new Promise((resolve) => {
      runner(component.data.properties, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });
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

// TODO(nick,paulo): re-add the catch branch.
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
