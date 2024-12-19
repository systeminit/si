import * as _ from "lodash-es";
import { Debug } from "../debug.ts";
import {
  failureExecution,
  Func,
  FunctionKind,
  ResultFailure,
  ResultSuccess,
  runCode,
} from "../function.ts";
import { ComponentWithGeometry, Geometry } from "../component.ts";
import { RequestCtx } from "../request.ts";

const debug = Debug("langJs:management");

export interface ManagementFunc extends Func {
  currentView: string;
  thisComponent: ComponentWithGeometry;
  components: {
    [key: string]: ComponentWithGeometry;
  };
}

export type ManagementFuncResult =
  | ManagementFuncResultSuccess
  | ManagementFuncResultFailure;

export interface ManagmentConnect {
  from: string;
  to: {
    component: string;
    socket: string;
  };
}

export interface ManagementOperations {
  create?: {
    [key: string]: {
      kind: string;
      properties?: object;
      geometry?: Geometry;
      parent?: string;
      connect?: ManagmentConnect[];
    };
  };
  update?: {
    [key: string]: {
      properties?: object;
      geometry?: { [key: string]: Geometry };
      parent?: string;
      connect: {
        add?: ManagmentConnect[];
        remove?: ManagmentConnect[];
      };
    };
  };
  actions?: {
    [key: string]: {
      add?: string[];
      remove?: string[];
    };
  };
}

export interface ManagementFuncResultSuccess extends ResultSuccess {
  health: "ok" | "error";
  operations?: ManagementOperations;
  message?: string;
}
export interface ManagementFuncResultFailure extends ResultFailure {}

async function execute(
  { executionId }: RequestCtx,
  { thisComponent, components, currentView }: ManagementFunc,
  code: string,
): Promise<ManagementFuncResult> {
  let managementResult: Record<string, unknown> | undefined | null;
  try {
    managementResult = await runCode(
      code,
      FunctionKind.Management,
      executionId,
      { thisComponent, components, currentView },
    );
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
        message:
          "Management functions must return an object with a status field",
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
        message:
          'Management functions must return a status of either "ok" or "error"',
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

const wrapCode = (code: string) => `
async function run(arg) {
  ${code}
  const returnValue = await main(arg);
  return returnValue;
}`;

export default {
  debug,
  execute,
  wrapCode,
};
