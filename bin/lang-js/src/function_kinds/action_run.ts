import Debug from "debug";
import {NodeVM} from "vm2";
import _ from "lodash";
import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
} from "../function";
import {RequestCtx} from "../request";

const debug = Debug("langJs:actionRun");

export interface ActionRunFunc extends Func {
  args: unknown;
}

export type ActionRunResult = ActionRunResultSuccess | ActionRunResultFailure;

export interface ActionRunResultSuccess extends ResultSuccess {
  payload: unknown;
  health: "ok" | "warning" | "error";
  message?: string;
}

export type ActionRunResultFailure = ResultFailure;

async function execute(
  vm: NodeVM,
  {executionId}: RequestCtx,
  {args}: ActionRunFunc,
  code: string,
): Promise<ActionRunResult> {
  let actionRunResult: Record<string, unknown>;
  try {
    const actionRunRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    actionRunResult = await new Promise((resolve) => {
      actionRunRunner(args, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });

    if (_.isUndefined(actionRunResult) || _.isNull(actionRunResult)) {
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
      !_.isString(actionRunResult["status"]) ||
      !["ok", "warning", "error"].includes(actionRunResult["status"])
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "ActionFieldWrongType",
          message:
            'The status field type must be either "ok", "warning" or "error"',
        },
      };
    }

    if (
      actionRunResult["status"] === "ok" &&
      !_.isUndefined(actionRunResult["message"])
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "ActionFieldWrongType",
          message:
            'The message field type must be undefined when status is "ok"',
        },
      };
    }

    if (
      actionRunResult["status"] !== "ok" &&
      !_.isString(actionRunResult["message"])
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "ActionFieldWrongType",
          message:
            'The message field type must be string when status is either "warning" or "error"',
        },
      };
    }

    return {
      protocol: "result",
      status: "success",
      executionId,
      error: actionRunResult.error as string | undefined,
      payload: actionRunResult.payload,
      health: actionRunResult.status as "ok" | "warning" | "error",
      message: actionRunResult.message as string | undefined,
    };
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
}

const wrapCode = (code: string, handle: string) => `
module.exports = function(arg, callback) {
  ${code}
  arg = Array.isArray(arg) ? arg : [arg];
  const resource = arg[0]?.properties?.resource?.payload ?? null;
  const returnValue = ${handle}(...arg, callback);
  if (returnValue instanceof Promise) {
    returnValue.then((data) => callback(data))
        .catch((err) => callback({
          status: "error",
          payload: resource,
          message: err.message,
  }));
  } else {
    callback(returnValue);
  }
};`

export default {
  debug,
  execute,
  wrapCode
}
