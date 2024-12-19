import * as _ from "lodash-es";
import { Debug } from "../debug.ts";
import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
  runCode,
} from "../function.ts";
import { RequestCtx } from "../request.ts";
import { FunctionKind } from "../function.ts";

const debug = Debug("langJs:actionRun");

export interface ActionRunFunc extends Func {
  args: unknown;
}

export type ActionRunResult = ActionRunResultSuccess | ActionRunResultFailure;

export interface ActionRunResultSuccess extends ResultSuccess {
  resourceId?: string | null;
  payload: unknown;
  health: "ok" | "warning" | "error";
  message?: string;
}

export type ActionRunResultFailure = ResultFailure;

async function execute(
  { executionId }: RequestCtx,
  { args }: ActionRunFunc,
  code: string,
): Promise<ActionRunResult> {
  try {
    const actionRunResult = await runCode(
      code,
      FunctionKind.ActionRun,
      executionId,
      args as Record<string, unknown>,
    );

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
      !_.isString(actionRunResult.status) ||
      !["ok", "warning", "error"].includes(actionRunResult.status as string)
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
      actionRunResult.status === "ok" &&
      !_.isUndefined(actionRunResult.message)
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
      actionRunResult.status !== "ok" &&
      !_.isString(actionRunResult.message)
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
      resourceId: actionRunResult.resourceId as string | undefined | null,
      payload: actionRunResult.payload,
      health: actionRunResult.status as "ok" | "warning" | "error",
      message: actionRunResult.message as string | undefined,
    };
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
}

const wrapCode = (code: string) => `
async function run(arg) {
  try {
    ${code}
    arg = Array.isArray(arg) ? arg : [arg];
    const resourceId = arg[0]?.properties?.si?.resourceId;
    const payload = arg[0]?.properties?.resource?.payload ?? null;

    const returnValue = await main(with_arg);
    return returnValue;
  } catch (err) {
    return {
              status: "error",
              payload,
              resourceId,
              message: err.message,
           }
  }
}`;

export default {
  debug,
  execute,
  wrapCode,
};
