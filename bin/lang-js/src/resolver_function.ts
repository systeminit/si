import Debug from "debug";
import _ from "lodash";
import { VM } from "vm2";
import { base64ToJs } from "./base64";
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

export enum FuncBackendResponseType {
  Action = "Action",
  Array = "Array",
  Boolean = "Boolean",
  Identity = "Identity",
  Integer = "Integer",
  Map = "Map",
  Object = "Object",
  Qualification = "Qualification",
  CodeGeneration = "CodeGeneration",
  Confirmation = "Confirmation",
  String = "String",
  Unset = "Unset",
  Json = "Json",
  Validation = "Validation",
}

export interface ResolverFunctionRequest extends RequestWithCode {
  // Should this be optional?
  component: ResolverComponent;
  responseType: FuncBackendResponseType;
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

export interface TypeCheckFailure {
  valid: false;
  message: string;
}

export interface TypeCheckSuccess {
  valid: true;
}

export type TypeCheckResult = TypeCheckFailure | TypeCheckSuccess;

const isArray = (value: unknown): TypeCheckResult =>
  _.isArray(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be an array." };

const isBoolean = (value: unknown): TypeCheckResult =>
  _.isBoolean(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be a boolean." };

const isInteger = (value: unknown): TypeCheckResult =>
  _.isInteger(value)
    ? { valid: true }
    : { valid: false, message: `Return type must be an integer.` };

// This check is not 100% valid because javascript does not distinguish
// between objects, arrays, functions and null in typeof checks. This
// could return true if the function returns another function.
const isObject = (value: unknown): TypeCheckResult =>
  typeof value === 'object' && _.isObject(value) && !_.isArray(value) && !_.isNull(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be an object." };

const isString = (value: unknown): TypeCheckResult =>
  _.isString(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be a string." };

const isCodeGeneration = (value: unknown): TypeCheckResult => {
  if (typeof value !== 'object' || !value) {
    return {
      valid: false,
      message: "CodeGenerations must return an object with 'format' and 'code' fields",
    }
  }

  if (!("format" in value) || !_.isString(value.format)) {
    return {
      valid: false,
      message: "The format field type must be a string",
    };
  }

  if (!("code" in value) || !_.isString(value.code)) {
    return {
      valid: false,
      message: "The code field type must be a string",
    };
  }

  return { valid: true };
};

const qualificationStatuses = ["warning", "failure", "success", "unknown"];
const isQualification = (value: unknown): TypeCheckResult => {
  if (typeof value !== 'object' || !value) {
    return { valid: false, message: "A qualification must return an object."};
  }

  if (!("result" in value) || !_.isString(value.result)) {
    return {
      valid: false,
      message: "Qualification result field type must be a string",
    };
  }

  if (!qualificationStatuses.includes(value.result)) {
    return {
      valid: false,
      message: "Qualification result must be one of 'success' | 'warning' | 'failure'",
    }
  }

  if (value.result !== 'success' && (!("message" in value) || !_.isString(value.message))) {
    return {
      valid: false,
      message: "The Qualification message field type must be a string, and must be present unless the status is success",
    };
  }

  return { valid: true };
};

const typeChecks: {
  [key in FuncBackendResponseType]?: (
    value: unknown
  ) => TypeCheckSuccess | TypeCheckFailure;
} = {
  [FuncBackendResponseType.Array]: isArray,
  [FuncBackendResponseType.Boolean]: isBoolean,
  [FuncBackendResponseType.Integer]: isInteger,
  [FuncBackendResponseType.Object]: isObject,
  [FuncBackendResponseType.String]: isString,
  [FuncBackendResponseType.Map]: isObject, // map funcs return js objects

  [FuncBackendResponseType.CodeGeneration]: isCodeGeneration,
  [FuncBackendResponseType.Qualification]: isQualification,
};

const nullables: { [key in FuncBackendResponseType]?: boolean } = {
  [FuncBackendResponseType.Array]: true,
  [FuncBackendResponseType.Boolean]: true,
  [FuncBackendResponseType.Integer]: true,
  [FuncBackendResponseType.Json]: true,
  [FuncBackendResponseType.Map]: true,
  [FuncBackendResponseType.Object]: true,
  [FuncBackendResponseType.String]: true,

  [FuncBackendResponseType.CodeGeneration]: false,
  [FuncBackendResponseType.Qualification]: false,
};

export async function executeResolverFunction(
  request: ResolverFunctionRequest
): Promise<void> {
  let code = base64ToJs(request.codeBase64);

  code = wrapCode(code, request.handler);
  debug({ code });

  const sandbox = createSandbox(
    FunctionKind.ResolverFunction,
    request.executionId
  );
  const vm = createNodeVm(sandbox);

  const result = await execute(vm, code, request);
  debug({ result });

  console.log(JSON.stringify(result));
}

async function execute(
  vm: VM,
  code: string,
  request: ResolverFunctionRequest
): Promise<ResolverFunctionResult> {
  const { executionId, component, responseType } = request;

  let resolverFunctionResult: Record<string, unknown>;
  try {
    const runner = vm.run(code);
    resolverFunctionResult = await new Promise((resolve) => {
      runner(component.data.properties, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }

  if (
    _.isUndefined(resolverFunctionResult) ||
    _.isNull(resolverFunctionResult)
  ) {
    vm.sandbox.console.debug("function returned undefined or null");
    if (nullables?.[responseType] === true) {
      return {
        protocol: "result",
        status: "success",
        executionId,
        data: resolverFunctionResult,
        unset: true,
      };
    } else {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "InvalidReturnType",
          message: "Return type cannot be null or undefined",
        },
      };
    }
  }

  const validationFunc = typeChecks?.[responseType] ?? undefined;
  if (validationFunc) {
    const validationResult = validationFunc(resolverFunctionResult);
    if (validationResult.valid === true) {
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
          message: validationResult.message,
        },
      };
    }
  }

  return {
    protocol: "result",
    status: "success",
    executionId,
    data: resolverFunctionResult,
    unset: false,
  };
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
