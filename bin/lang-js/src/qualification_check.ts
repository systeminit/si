import Debug from "debug";
import _ from "lodash";
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
import { Component } from "./component";

const debug = Debug("langJs:qualificationCheck");

export interface QualificationCheckRequest extends Request {
  handler: string;
  component: QualificationComponent;
  codeBase64: string;
}

export interface Code {
  format: string;
  code: string;
}

export interface QualificationComponent {
  data: Component;
  codes: Array<Code>;
}

export type QualificationCheckResult =
  | QualificationCheckResultSuccess
  | QualificationCheckResultFailure;

export interface QualificationCheckResultSuccess extends ResultSuccess {
  qualified: boolean;
  title?: string;
  link?: string;
  subChecks?: Array<{
    status: "Success" | "Failure" | "Unknown",
    description: string,
  }>,
  message?: string;
}

export interface QualificationCheckResultFailure extends ResultFailure {
  qualified?: never;
  message?: never;
}

export async function executeQualificationCheck(
  request: QualificationCheckRequest
): Promise<void> {
  let code = base64Decode(request.codeBase64);
  debug({ code });

  code = wrapCode(code, request.handler);
  debug({ code });

  const sandbox = createSandbox(
    FunctionKind.QualificationCheck,
    request.executionId
  );
  const vm = createNodeVm(sandbox);

  const result = await execute(vm, code, request.component, request.executionId);
  debug({ result });

  console.log(JSON.stringify(result));
}

async function execute(
  vm: NodeVM,
  code: string,
  component: QualificationComponent,
  executionId: string
): Promise<QualificationCheckResult> {
  let qualificationCheckResult: any;
  try {
    const qualificationCheckRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    qualificationCheckResult = await new Promise((resolve) => {
      qualificationCheckRunner(component, (resolution: unknown) => resolve(resolution));
    });
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
    !_.isString(qualificationCheckResult["message"]) &&
    !_.isUndefined(qualificationCheckResult["message"]) &&
    !_.isNull(qualificationCheckResult["message"])
  ) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "MessageFieldWrongType",
        message: "The message field type must be string, null, or undefined",
      },
    };
  }

  const result: QualificationCheckResultSuccess = {
    protocol: "result",
    status: "success",
    executionId,
    title: qualificationCheckResult["title"],
    link: qualificationCheckResult["link"],
    subChecks: qualificationCheckResult["subChecks"],
    qualified: qualificationCheckResult["qualified"],
  };
  if (qualificationCheckResult["message"]) {
    result.message = qualificationCheckResult["message"];
  }
  return result;
}

// TODO(paulo): handle promise exceptions in a better way, VM2 sadly doesn't have a catch-like callback
function wrapCode(code: string, handle: string): string {
  const wrapped = `module.exports = function(component, callback) {
    ${code}
    const returnValue = ${handle}(component, callback);
    if (returnValue instanceof Promise) {
      returnValue.then((data) => callback(data))
        .catch((err) => {
          const message = "Uncaught throw in a promise, inside function ${handle}: " + err.message;
          callback({
            qualified: false,
            message
          })
        });
    } else {
      callback(returnValue);
    }
  };`;
  return wrapped;
}
