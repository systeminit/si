import Debug from "debug";
import { NodeVM } from "vm2";
import * as _ from "lodash-es";
import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
} from "../function";
import { RequestCtx } from "../request";

const debug = Debug("langJs:reconciliation");

export interface ReconciliationFunc extends Func {
  args: unknown;
}

export type ReconciliationResult =
  | ReconciliationResultSuccess
  | ReconciliationResultFailure;

export interface ReconciliationResultSuccess extends ResultSuccess {
  updates: { [key: string]: unknown };
  actions: string[];
  message: string | undefined;
}

export type ReconciliationResultFailure = ResultFailure;

async function execute(
  vm: NodeVM,
  { executionId }: RequestCtx,
  { args }: ReconciliationFunc,
  code: string,
): Promise<ReconciliationResult> {
  let reconciliationResult: Record<string, unknown>;
  try {
    const reconciliationRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    reconciliationResult = await new Promise((resolve) => {
      reconciliationRunner(args, (resolution: Record<string, unknown>) => resolve(resolution));
    });

    if (_.isUndefined(reconciliationResult) || _.isNull(reconciliationResult)) {
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

    if (!_.isObject(reconciliationResult.updates)) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "ReconciliationFieldWrongType",
          message: "The updates field type must be an object",
        },
      };
    }

    if (
      !_.isArray(reconciliationResult.actions)
      || reconciliationResult.actions.some((v) => typeof v !== "string")
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "ReconciliationFieldWrongType",
          message: "The actions field type must be an array of strings",
        },
      };
    }

    return {
      protocol: "result",
      status: "success",
      executionId,
      error: reconciliationResult.error as string | undefined,
      updates: reconciliationResult.updates as { [key: string]: unknown },
      actions: reconciliationResult.actions as string[],
      message: reconciliationResult.message as string | undefined,
    };
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
}

const wrapCode = (code: string, handle: string) => `
module.exports = function(arg, callback) {
  ${code}
  const returnValue = ${handle}(arg, callback);
  if (returnValue instanceof Promise) {
    returnValue.then((data) => callback(data))
        .catch((err) => callback({
    message: err.message,
    updates: {},
    actions: []
  }));
  } else {
    callback(returnValue);
  }
};`;

export default {
  debug,
  execute,
  wrapCode,
};
