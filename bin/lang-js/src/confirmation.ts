import Debug from "debug";
import _ from "lodash";
import { base64Decode } from "./base64";
import { NodeVM } from "vm2";
import {
  failureExecution,
  FunctionKind,
  RequestWithCode,
  ResultFailure,
  ResultSuccess,
} from "./function";
import { createSandbox } from "./sandbox";
import { createNodeVm } from "./vm";
import { Component } from "./component";

const debug = Debug("langJs:confirmation");

export interface ConfirmationRequest extends RequestWithCode {
  component: Component;
}

export interface Code {
  format: string;
  code: string;
}

export type ConfirmationResult =
  | ConfirmationResultSuccess
  | ConfirmationResultFailure;

export interface ConfirmationResultSuccess extends ResultSuccess {
  success: boolean;
  recommendedActions?: string[];
  message?: string;
}

export interface ConfirmationResultFailure extends ResultFailure {
  success?: never;
  recommendedActions?: never;
  message?: never;
}

export async function executeConfirmation(
  request: ConfirmationRequest
): Promise<void> {
  let code = base64Decode(request.codeBase64);
  debug({ code });

  code = wrapCode(code, request.handler);
  debug({ code });

  const sandbox = createSandbox(FunctionKind.Confirmation, request.executionId);
  const vm = createNodeVm(sandbox);

  const result = await execute(
    vm,
    code,
    request.component,
    request.executionId
  );
  debug({ result });

  console.log(JSON.stringify(result));
}

async function execute(
  vm: NodeVM,
  code: string,
  component: Component,
  executionId: string
): Promise<ConfirmationResult> {
  let confirmationResult: Record<string, unknown>;
  try {
    const confirmationRunner = vm.run(code);
    // Node(paulo): NodeVM doesn't support async rejection, we need a better way of handling it
    confirmationResult = await new Promise((resolve) => {
      confirmationRunner(component, (resolution: Record<string, unknown>) =>
        resolve(resolution)
      );
    });
    debug({ result: JSON.stringify(confirmationResult) });
  } catch (err) {
    return failureExecution(err, executionId);
  }

  if (_.isUndefined(confirmationResult) || _.isNull(confirmationResult)) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "InvalidReturnType",
        message: "return type must not be unset",
      },
    };
  }

  if (!_.isBoolean(confirmationResult["success"])) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "InvalidReturnType",
        message: "success field must be a boolean",
      },
    };
  }

  if (confirmationResult["success"]) {
    if (
      !(
        _.isUndefined(confirmationResult["recommendedActions"]) ||
        (_.isArray(confirmationResult["recommendedActions"]) &&
          _.isEmpty(confirmationResult["recommendedActions"]))
      )
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "InvalidReturnType",
          message:
            "recommendedActions field must undefined or an empty array on success",
        },
      };
    }
  } else {
    if (!_.isArray(confirmationResult["recommendedActions"])) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "InvalidReturnType",
          message:
            "recommendedActions field must be an array of strings on failure",
        },
      };
    }

    if (
      confirmationResult["recommendedActions"].some(
        (field) => !_.isString(field)
      )
    ) {
      return {
        protocol: "result",
        status: "failure",
        executionId,
        error: {
          kind: "InvalidReturnType",
          message:
            "recommendedActions field must be an array of strings on failure",
        },
      };
    }
  }

  if (
    !_.isUndefined(confirmationResult["message"]) &&
    !_.isString(confirmationResult["message"])
  ) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "InvalidReturnType",
        message: "message field must be either unset or a string on failure",
      },
    };
  }

  const result: ConfirmationResultSuccess = {
    protocol: "result",
    status: "success",
    executionId,
    success: confirmationResult["success"],
    recommendedActions: confirmationResult["recommendedActions"],
    message: confirmationResult["failure"] as string,
  };
  return result;
}

// TODO(paulo): handle promise exceptions in a better way, VM2 sadly doesn't have a catch-like callback
function wrapCode(code: string, handle: string): string {
  const wrapped = `module.exports = function(component, callback) {
    ${code}
    const returnValue = ${handle}(component);
    if (returnValue instanceof Promise) {
      returnValue.then((data) => callback(data))
        .catch((err) => {
          callback({
            success: false,
            message: err.message,
          })
        });
    } else {
      callback(returnValue);
    }
  };`;
  return wrapped;
}
