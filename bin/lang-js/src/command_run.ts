import Debug from "debug";
import { base64Decode } from "./base64";
import { NodeVM } from "vm2";
import {
  failureExecution,
  FunctionKind,
  Request, RequestWithCode,
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
  created?: Record<string, unknown>;
  updated?: Record<string, unknown>;
}
export type CommandRunResultFailure = ResultFailure;

export async function executeCommandRun(
  request: CommandRunRequest
): Promise<void> {
  let code = base64Decode(request.codeBase64);
  debug({ code });

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
  args: unknown,
): Promise<CommandRunResult> {
  let commandRunResult: Record<string, unknown>;
  try {
    const commandRunRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    commandRunResult = await new Promise((resolve) => {
      commandRunRunner(
        args,
        (resolution: Record<string, unknown>) => resolve(resolution)
      );
    });

    const result: CommandRunResultSuccess = {
      protocol: "result",
      status: "success",
      executionId,
      error: commandRunResult?.error as string,
      updated: commandRunResult?.updated as Record<string, unknown> | undefined,
      created: commandRunResult?.created as Record<string, unknown> | undefined,
    };
    return result;
  } catch (err) {
    return failureExecution(err, executionId);
  }
}

function wrapCode(code: string, handle: string): string {
  const wrapped = `module.exports = function(args, callback) {
    ${code}
    const arguments = Array.isArray(args) ? args : [args];
    const returnValue = ${handle}(...arguments, callback);
    if (returnValue instanceof Promise) {
      returnValue.then((data) => callback(data))
        .catch((err) => {
          callback({
            error: err.message,
          })
        });
    } else {
      callback(returnValue);
    }
  };`;
  return wrapped;
}
