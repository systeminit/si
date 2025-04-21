import * as _ from "https://deno.land/x/lodash_es@v0.0.2/mod.ts";
import { Debug } from "../debug.ts";
import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
} from "../function.ts";
import { runCode } from "../execution.ts";
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
  timeout: number,
): Promise<ActionRunResult> {
  try {
    const actionRunResult = await runCode(
      code,
      "siMain",
      FunctionKind.ActionRun,
      executionId,
      timeout,
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

// TODO(nick,scott,paul): does this even need a wrapper at all? Only actions have a wrapper like
// this at the time of writing.
const wrapCode = (code: string, handler: string) => `
${code}
async function siMain(arg) {
  let payload = null;
  let resourceId = null;
  try {
    resourceId = arg?.properties?.si?.resourceId;
    payload = arg?.properties?.resource?.payload ?? null;

    const returnValue = await ${handler}(arg);
    return returnValue;
  } catch (err) {
    return {
              status: "error",
              payload,
              resourceId,
              message: err.message,
           }
  }
}
export { siMain };
`;

export default {
  debug,
  execute,
  wrapCode,
};
