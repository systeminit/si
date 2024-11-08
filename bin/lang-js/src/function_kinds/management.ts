import * as _ from "lodash-es";
import { NodeVM } from "vm2";
import { Debug } from "../debug";
import {
  failureExecution,
  Func,
  ResultFailure,
  ResultSuccess,
} from "../function";
import { ComponentWithGeometry, Geometry } from "../component";
import { RequestCtx } from "../request";

const debug = Debug("langJs:management");

export interface ManagementFunc extends Func {
  thisComponent: ComponentWithGeometry;
  components: {
    [key: string]: ComponentWithGeometry;
  }
}

export type ManagementFuncResult =
    | ManagementFuncResultSuccess
    | ManagementFuncResultFailure;

export interface ManagmentConnect {
  from: string,
  to: {
    component: string;
    socket: string;
  }
}

export interface ManagementOperations {
  create?: { [key: string]: {
    kind: string;
    properties?: object;
    geometry?: Geometry;
    parent?: string;
    connect?: ManagmentConnect[],
  } };
  update?: { [key: string]: {
    properties?: object;
    geometry?: Geometry;
    parent?: string;
    connect: {
      add?: ManagmentConnect[],
      remove?: ManagmentConnect[],
    }
  } };
  actions?: {
    [key: string]: {
      add?: string[],
      remove?: string[],
    }
  };
}

export interface ManagementFuncResultSuccess extends ResultSuccess {
  health: "ok" | "error",
  operations?: ManagementOperations,
  message?: string;
}
export interface ManagementFuncResultFailure extends ResultFailure { }

async function execute(
  vm: NodeVM,
  { executionId }: RequestCtx,
  { thisComponent, components }: ManagementFunc,
  code: string,
): Promise<ManagementFuncResult> {
  let managementResult: Record<string, unknown> | undefined | null;
  try {
    const runner = vm.run(code);
    managementResult = await new Promise((resolve) => {
      runner({ thisComponent, components }, (resolution: Record<string, unknown>) => resolve(resolution));
    });
  } catch (err) {
    return failureExecution(err as Error, executionId);
  }

  const status = managementResult?.status;
  if (!status || typeof status !== "string") {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "InvalidReturnType",
        message: "Management functions must return an object with a status field",
      },
    };
  }

  if (!(["ok", "error"].includes(status))) {
    return {
      protocol: "result",
      status: "failure",
      executionId,
      error: {
        kind: "InvalidReturnType",
        message: "Management functions must return a status of either \"ok\" or \"error\"",
      },
    };
  }

  return {
    protocol: "result",
    status: "success",
    executionId,
    health: status as "ok" | "error",
    operations: managementResult?.ops as ManagementOperations | undefined,
    message: managementResult?.message as string | undefined,
  };
}

// Should we wrap this in a try/catch ?
const wrapCode = (code: string, handle: string) => `
module.exports = function(input, callback) {
  ${code}
  const returnValue = ${handle}(input);
  if (returnValue instanceof Promise) {
    returnValue.then((data) => callback(data))
  } else {
    callback(returnValue);
  }
};`;

export default {
  debug,
  execute,
  wrapCode,
};
