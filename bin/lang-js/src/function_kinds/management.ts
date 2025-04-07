import * as _ from "https://deno.land/x/lodash_es@v0.0.2/mod.ts";
import { Debug } from "../debug.ts";
import {
  failureExecution,
  Func,
  FunctionKind,
  ResultFailure,
  ResultSuccess,
} from "../function.ts";
import { runCode } from "../execution.ts";
import { ComponentWithGeometry, Geometry } from "../component.ts";
import { RequestCtx } from "../request.ts";

const debug = Debug("langJs:management");

export interface ManagementFunc extends Func {
  currentView: string;
  thisComponent: ComponentWithGeometry;
  components: {
    [key: string]: ComponentWithGeometry;
  };
  variantSocketMap: Record<SchemaName, number>;
}

export type ManagementFuncResult =
  | ManagementFuncResultSuccess
  | ManagementFuncResultFailure;

// These types help us know what various strings actually reference, even though they are just strings.

/// Name of a schema.
export type SchemaName = string;

/// A ULID component ID referencing a component that already exists.
export type ComponentId = string;

/// A name referencing a new component being created (returned by ManagementFunc.ops.create),
/// or an existing component under management (sent to ManagementFunc.components).
///
/// If a new component has the same name as an existing component, the new component takes priority.
export type ManagedComponentRef = string;

/// A name referencing a managed component (or one that is being created) or a ULID component ID.
///
/// If a new or managed component's name is the same as a component's ULID, the new or managed
/// component takes priority.
export type ComponentRef = ManagedComponentRef | ComponentId;

/// A name referencing a socket on a component.
export type SocketName = string;

/// A reference to a socket on a component.
export interface SocketRef {
  component: ComponentRef;
  socket: SocketName;
}

/// A reference to a socket on an existing component on the graph. Always a component ID.
export interface SocketRefAndValue extends SocketRef {
  component: ComponentId;
  socket: SocketName;
  value: any;
}

/// A connection to or from a component.
///
/// If the to field is a SocketName (string), it's an outgoing connection.
/// If the from field is a SocketName (string), it's an incoming connection.
export type ManagementConnect =
  | { from: SocketRef; to: SocketName }
  | { from: SocketName; to: SocketRef };

export interface ManagementCreate {
  [key: string]: {
    kind: SchemaName;
    properties?: object;
    geometry?: Geometry | { [key: string]: Geometry };
    parent?: ComponentRef;
    connect?: ManagementConnect[];
  };
}

export interface ManagementOperations {
  create?: ManagementCreate;
  update?: {
    [key: ManagedComponentRef]: {
      properties?: object;
      geometry?: { [key: string]: Geometry };
      parent?: ManagedComponentRef; // TODO allow external parent?
      connect: {
        add?: ManagementConnect[];
        remove?: ManagementConnect[];
      };
    };
  };
  actions?: {
    [key: ManagedComponentRef]: {
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
  {
    thisComponent,
    components,
    currentView,
    variantSocketMap,
    handler,
  }: ManagementFunc,
  code: string,
  timeout: number,
): Promise<ManagementFuncResult> {
  let managementResult: Record<string, unknown> | undefined | null;
  try {
    managementResult = await runCode(
      code,
      handler,
      FunctionKind.Management,
      executionId,
      timeout,
      {
        thisComponent,
        components,
        currentView,
        variantSocketMap,
      },
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

const wrapCode = (code: string, handler: string) => `
  ${code}
  export { ${handler} };
`;

export default {
  debug,
  execute,
  wrapCode,
};
