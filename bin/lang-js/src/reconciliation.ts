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

const debug = Debug("langJs:reconciliation");

export interface ReconciliationRequest extends RequestWithCode {
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

export async function executeReconciliation(
  request: ReconciliationRequest
): Promise<void> {
  let code = base64ToJs(request.codeBase64);

  code = wrapCode(code, request.handler);
  debug({ code });

  const sandbox = createSandbox(
    FunctionKind.Reconciliation,
    request.executionId
  );
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
): Promise<ReconciliationResult> {
  let reconciliationResult: Record<string, unknown>;
  try {
    const reconciliationRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    reconciliationResult = await new Promise((resolve) => {
      reconciliationRunner(args, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
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

    if (!_.isObject(reconciliationResult["updates"])) {
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
      !_.isArray(reconciliationResult["actions"]) ||
      reconciliationResult["actions"].some((v) => typeof v !== "string")
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

    const result: ReconciliationResultSuccess = {
      protocol: "result",
      status: "success",
      executionId,
      error: reconciliationResult.error as string | undefined,
      updates: reconciliationResult.updates as { [key: string]: unknown },
      actions: reconciliationResult.actions as string[],
      message: reconciliationResult.message as string | undefined,
    };
    return result;
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }
}

function wrapCode(code: string, handle: string): string {
  const wrapped = `module.exports = function(arg, callback) {
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
  return wrapped;
}
