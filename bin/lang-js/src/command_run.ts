import Debug from "debug";
import { base64Decode } from "./base64";
import { NodeVM } from "vm2";
import {
  failureExecution,
  FunctionKind,
  Request,
  ResultFailure,
  ResultSuccess,
} from "./function";
import { createSandbox } from "./sandbox";
import { createNodeVm } from "./vm";

const debug = Debug("langJs:commandRun");

export interface CommandRunRequest extends Request {
  handler: string;
  codeBase64: string;
}

export type CommandRunResult =
  | CommandRunResultSuccess
  | CommandRunResultFailure;

export type CommandRunResultSuccess = ResultSuccess;
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

  const result = await execute(vm, code, request.executionId);
  debug({ result });

  console.log(JSON.stringify(result));
}

async function execute(
  vm: NodeVM,
  code: string,
  executionId: string
): Promise<CommandRunResult> {
  let _commandRunResult: Record<string, unknown>;
  try {
    const commandRunRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    _commandRunResult = await new Promise((resolve) => {
      commandRunRunner((resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });

    const result: CommandRunResultSuccess = {
      protocol: "result",
      status: "success",
      executionId,
    };
    return result;
  } catch (err) {
    return failureExecution(err, executionId);
  }
}

function wrapCode(code: string, handle: string): string {
  const wrapped = `module.exports = function(callback) {
    ${code}
    const returnValue = ${handle}(callback);
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
