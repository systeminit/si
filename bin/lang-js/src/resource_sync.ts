import Debug from "debug";
import _ from "lodash";
import { VM, VMScript } from "vm2";
import { base64Decode } from "./base64";
import {
  failureExecution,
  FunctionKind,
  Request,
  ResultFailure,
  ResultSuccess,
} from "./function";
import { createSandbox } from "./sandbox";
import { createVm } from "./vm";

const debug = Debug("langJs:resourceSync");

export interface ResourceSyncRequest extends Request {
  handler: string;
  component: Component;
  codeBase64: string;
}

export interface Component {
  name: string;
  // TODO(fnichol): Highly, highly, highly TBD!
  properties: Record<string, unknown>;
}

export type ResourceSyncResult =
  | ResourceSyncResultSuccess
  | ResourceSyncResultFailure;

export interface ResourceSyncResultSuccess extends ResultSuccess {
  something?: string;
}

export interface ResourceSyncResultFailure extends ResultFailure {
  something?: never;
}

export function executeResourceSync(request: ResourceSyncRequest): void {
  const code = base64Decode(request.codeBase64);
  debug({ code });
  const compiledCode = new VMScript(wrapCode(code, request.handler, request.component)).compile();
  debug({ code: compiledCode.code });
  const sandbox = createSandbox(FunctionKind.ResourceSync, request.executionId);
  const vm = createVm(sandbox);

  const result = execute(vm, compiledCode, request.executionId);
  debug({ result });

  console.log(JSON.stringify(result));
}

function execute(
  vm: VM,
  code: VMScript,
  executionId: string
): ResourceSyncResult {
  let resourceSyncResult;
  try {
    resourceSyncResult = vm.run(code);
  } catch (err) {
    return failureExecution(err, executionId);
  }

  if (_.isUndefined(resourceSyncResult) || _.isNull(resourceSyncResult)) {
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

  // TODO(fnichol): minimum checking of other result fields here...

  const result: ResourceSyncResultSuccess = {
    protocol: "result",
    status: "success",
    executionId,
  };
  return result;
}

function wrapCode(code: string, handle: string, component: Component): string {
  return code + `\n${handle}(${JSON.stringify(component)});\n`;
}
