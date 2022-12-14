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

export enum FuncBackendResponseType {
  Array = "Array",
  Boolean = "Boolean",
  Identity = "Identity",
  Integer = "Integer",
  Map = "Map",
  PropObject = "PropObject",
  Qualification = "Qualification",
  CodeGeneration = "CodeGeneration",
  Confirmation = "Confirmation",
  String = "String",
  Unset = "Unset",
  Json = "Json",
  Validation = "Validation",
  Workflow = "Workflow",
  Command = "Command",
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

/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isArray = (value: any): TypeCheckResult =>
  _.isArray(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be an array." };

/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isBoolean = (value: any): TypeCheckResult =>
  _.isBoolean(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be a boolean." };

/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isInteger = (value: any): TypeCheckResult =>
  _.isInteger(value)
    ? { valid: true }
    : { valid: false, message: `Return type must be an integer.` };

// This check is not 100% good because javascript 'objects' are annoyingly
// weird.
/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isObject = (value: any): TypeCheckResult =>
  _.isObject(value) && !_.isArray(value) && !_.isNull(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be an object." };

/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isString = (value: any): TypeCheckResult =>
  _.isString(value)
    ? { valid: true }
    : { valid: false, message: "Return type must be a string." };

/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isCodeGeneration = (value: any): TypeCheckResult => {
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

/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
const isValidation = (value: any): TypeCheckResult => {
  if (!("valid" in value) || !(typeof value.valid === "boolean")) {
    return {
      valid: false,
      message: "The 'valid' field type must be a boolean",
    };
  }

  if (
    !(typeof value.message === "undefined") &&
    !(typeof value.message === "string")
  ) {
    return {
      valid: false,
      message: "The 'message' field must be a string or undefined",
    };
  }

  return { valid: true };
};

const typeChecks: {
  [key in FuncBackendResponseType]?: (
    /* eslint-disable-next-line @typescript-eslint/no-explicit-any */
    value: any
  ) => TypeCheckSuccess | TypeCheckFailure;
} = {
  [FuncBackendResponseType.Array]: isArray,
  [FuncBackendResponseType.Boolean]: isBoolean,
  [FuncBackendResponseType.Integer]: isInteger,
  [FuncBackendResponseType.PropObject]: isObject,
  [FuncBackendResponseType.String]: isString,
  [FuncBackendResponseType.Map]: isObject, // map funcs return js objects

  [FuncBackendResponseType.CodeGeneration]: isCodeGeneration,
  [FuncBackendResponseType.Validation]: isValidation,
};

const nullables: { [key in FuncBackendResponseType]?: boolean } = {
  [FuncBackendResponseType.Array]: true,
  [FuncBackendResponseType.Boolean]: true,
  [FuncBackendResponseType.Integer]: true,
  [FuncBackendResponseType.PropObject]: true,
  [FuncBackendResponseType.String]: true,
  [FuncBackendResponseType.Map]: true,

  [FuncBackendResponseType.CodeGeneration]: false,
  [FuncBackendResponseType.Validation]: false,
};

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
    return failureExecution(err, executionId);
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
