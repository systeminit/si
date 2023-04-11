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

const debug = Debug("langJs:workflowResolve");

export interface WorkflowResolveRequest extends RequestWithCode {
  args: unknown;
}

export type WorkflowResolveResult =
  | WorkflowResolveResultSuccess
  | WorkflowResolveResultFailure;

export interface WorkflowResolveResultSuccess extends ResultSuccess {
  name: string;
  kind: string;
  steps: unknown;
  args: unknown;
}

export type WorkflowResolveResultFailure = ResultFailure;

export async function executeWorkflowResolve(
  request: WorkflowResolveRequest
): Promise<void> {
  let code = base64ToJs(request.codeBase64);

  code = wrapCode(code, request.handler);
  debug({ code });

  const sandbox = createSandbox(
    FunctionKind.WorkflowResolve,
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
): Promise<WorkflowResolveResult> {
  let workflowResolveResult: Record<string, unknown>;
  try {
    const workflowResolveRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    workflowResolveResult = await new Promise((resolve) => {
      workflowResolveRunner(args, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });

    if (
      _.isUndefined(workflowResolveResult) ||
      _.isNull(workflowResolveResult)
    ) {
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

    if (!_.isString(workflowResolveResult["name"])) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "WorkflowFieldWrongType",
          message: "The name field type must be string",
        },
      };
    }

    if (!_.isString(workflowResolveResult["kind"])) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "WorkflowFieldWrongType",
          message: "The kind field type must be string",
        },
      };
    }

    // TODO: validate steps and args

    const result: WorkflowResolveResultSuccess = {
      protocol: "result",
      status: "success",
      executionId,
      name: workflowResolveResult.name as string,
      kind: workflowResolveResult.kind as string,
      steps: workflowResolveResult.steps,
      args: workflowResolveResult.args,
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
    const returnValue = ${handle}(...arg, callback);
    if (returnValue instanceof Promise) {
      returnValue.then((data) => callback(data))
        .catch((err) => {
          const message = "Uncaught throw in a promise, in function ${handle}: " + err.message;
          callback({
            message,
          })
        });
    } else {
      callback(returnValue);
    }
  };`;
  return wrapped;
}
