import Debug from "debug";
import { base64ToJs } from "./base64";
import { NodeVM } from "vm2";
import _ from "lodash";
import {
  failureExecution,
  FunctionKind,
  RequestWithCode,
  ResultFailure,
  ResultSuccess,
} from "./function";
import { createSandbox } from "./sandbox";
import { createNodeVm } from "./vm";

const debug = Debug("langJs:commandRun");

export interface CommandRunRequest extends RequestWithCode {
  args: unknown;
}

export type CommandRunResult =
  | CommandRunResultSuccess
  | CommandRunResultFailure;

export interface CommandRunResultSuccess extends ResultSuccess {
  value: unknown;
  health: "ok" | "warning" | "error";
  message?: string;
}
export type CommandRunResultFailure = ResultFailure;

export async function executeCommandRun(
  request: CommandRunRequest
): Promise<void> {
  let code = base64ToJs(request.codeBase64);

  code = wrapCode(code, request.handler);
  debug({ code });

  const sandbox = createSandbox(FunctionKind.CommandRun, request.executionId);
  const vm = createNodeVm(sandbox);

  const result = await execute(vm, code, request.executionId, request.args);
  debug({ result });

  console.log(JSON.stringify(result));
}

async function execute(
  vm: NodeVM,
  code: string,
  executionId: string,
  args: unknown
): Promise<CommandRunResult> {
  let commandRunResult: Record<string, unknown>;
  try {
    const commandRunRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    commandRunResult = await new Promise((resolve) => {
      commandRunRunner(args, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });

    if (_.isUndefined(commandRunResult) || _.isNull(commandRunResult)) {
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

    if (
      !_.isString(commandRunResult["status"]) ||
      !["ok", "warning", "error"].includes(commandRunResult["status"])
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "WorkflowFieldWrongType",
          message:
            'The status field type must be either "ok", "warning" or "error"',
        },
      };
    }

    if (
      commandRunResult["status"] === "ok" &&
      !_.isUndefined(commandRunResult["message"])
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "WorkflowFieldWrongType",
          message:
            'The message field type must be undefined when status is "ok"',
        },
      };
    }

    if (
      commandRunResult["status"] !== "ok" &&
      !_.isString(commandRunResult["message"])
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "WorkflowFieldWrongType",
          message:
            'The message field type must be string when status is either "warning" or "error"',
        },
      };
    }

    const result: CommandRunResultSuccess = {
      protocol: "result",
      status: "success",
      executionId,
      error: commandRunResult.error as string | undefined,
      value: commandRunResult.value,
      health: commandRunResult.status as "ok" | "warning" | "error",
      message: commandRunResult.message as string | undefined,
    };
    return result;
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
}

function wrapCode(code: string, handle: string): string {
  const wrapped = `module.exports = function(arg, callback) {
    ${code}
    arg = Array.isArray(arg) ? arg : [arg];
    const resource = arg[0]?.properties?.resource?.value ?? null;
    const returnValue = ${handle}(...arg, callback);
    if (returnValue instanceof Promise) {
      returnValue.then((data) => callback(data))
          .catch((err) => callback({
            status: "error",
            value: resource,
      	    message: err.message,
	  }));
    } else {
      callback(returnValue);
    }
  };`;
  return wrapped;
}
