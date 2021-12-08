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

const debug = Debug("langJs:qualificationCheck");

export interface QualificationCheckRequest extends Request {
  handler: string;
  component: Component;
  codeBase64: string;
}

export interface Component {
  name: string;
  // TODO(fnichol): Highly, highly, highly TBD!
  properties: Record<string, unknown>;
}

export type QualificationCheckResult =
  | QualificationCheckResultSuccess
  | QualificationCheckResultFailure;

export interface QualificationCheckResultSuccess extends ResultSuccess {
  qualified: boolean;
  output?: string;
}

export interface QualificationCheckResultFailure extends ResultFailure {
  qualified?: never;
  output?: never;
}

export function executeQualificationCheck(
  request: QualificationCheckRequest
): void {
  const code = base64Decode(request.codeBase64);
  debug({ code });
  const compiledCode = new VMScript(
    wrapCode(code, request.handler, request.component)
  ).compile();
  debug({ code: compiledCode.code });
  const sandbox = createSandbox(
    FunctionKind.QualificationCheck,
    request.executionId
  );
  const vm = createVm(sandbox);

  const result = execute(vm, compiledCode, request.executionId);
  debug({ result });

  console.log(JSON.stringify(result));
}

function execute(
  vm: VM,
  code: VMScript,
  executionId: string
): QualificationCheckResult {
  let qualificationCheckResult;
  try {
    qualificationCheckResult = vm.run(code);
  } catch (err) {
    return failureExecution(err, executionId);
  }

  if (
    _.isUndefined(qualificationCheckResult) ||
    _.isNull(qualificationCheckResult)
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

  if (!_.isBoolean(qualificationCheckResult["qualified"])) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "QualifiedFieldWrongType",
        message: "The qualified field type must be boolean",
      },
    };
  }
  if (
    !_.isString(qualificationCheckResult["output"]) &&
    !_.isUndefined(qualificationCheckResult["output"]) &&
    !_.isNull(qualificationCheckResult["output"])
  ) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "OutputFieldWrongType",
        message: "The output field type must be string, null, or undefined",
      },
    };
  }

  const result: QualificationCheckResultSuccess = {
    protocol: "result",
    status: "success",
    executionId,
    qualified: qualificationCheckResult["qualified"],
  };
  if (qualificationCheckResult["output"]) {
    result.output = qualificationCheckResult["output"];
  }
  return result;
}

function wrapCode(code: string, handle: string, component: Component): string {
  return code + `\n${handle}(${JSON.stringify(component)});\n`;
}
