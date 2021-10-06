import { compileCode, createVm } from "./vm";
import { createSandbox } from "./sandbox";
import _ from "lodash";

export interface RemoteFunctionRequestResolver {
  kind: "resolver";
  code: string;
  containerImage: string;
  containerTag: string;
}

export type RemoteFunctionRequest = RemoteFunctionRequestResolver;

export interface RemoteFunctionOutputLine {
  stream: "stdout" | "stderr";
  level: "debug" | "info" | "warn" | "error";
  group?: string;
  message: string;
  data: unknown;
  timestamp: number;
}

export interface RemoteFunctionResultFailure {
  status: "failure";
  kind: string;
  error: {
    message: string;
    name: string;
  };
  data?: never;
  unset?: never;
}

export interface RemoteFunctionResultResolver {
  status: "success";
  kind: "resolver";
  error?: never;
  data?: unknown;
  unset: boolean;
}

export type RemoteFunctionResult =
  | RemoteFunctionResultResolver
  | RemoteFunctionResultFailure;

export function executeRemoteFunction(request: RemoteFunctionRequest) {
  const code = compileCode(request.code);
  const sandbox = createSandbox(request.kind);
  const vm = createVm(request.kind, _.cloneDeep(sandbox));
  let functionResult;
  let result: RemoteFunctionResult;
  try {
    functionResult = vm.run(code);
  } catch (e) {
    result = {
      status: "failure",
      kind: request.kind,
      error: {
        message: e.message,
        name: e.name,
      },
    };
  }
  if (!result) {
    if (_.isUndefined(functionResult)) {
      sandbox.console.log("function returned undefined");
      result = {
        status: "success",
        kind: request.kind,
        unset: true,
      };
    } else if (
      _.isString(functionResult) ||
      _.isNumber(functionResult) ||
      _.isBoolean(functionResult) ||
      _.isPlainObject(functionResult) ||
      _.isArray(functionResult) ||
      _.isNull(functionResult)
    ) {
      result = {
        status: "success",
        kind: request.kind,
        data: functionResult,
        unset: false,
      };
    } else {
      result = {
        status: "failure",
        kind: request.kind,
        error: {
          message:
            "Only strings, numbers, booleans, objects, arrays and null are allowed!",
          name: "InvalidReturnType",
        },
      };
    }
  }
  console.log(JSON.stringify(result));
  return result;
}
