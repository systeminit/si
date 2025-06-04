import {
  ActionId,
  ActionKind,
  ActionPrototypeId,
  ActionResultState,
} from "@/api/sdf/dal/action";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";

export type FuncRunId = string;
export type FuncRunLogId = string;
export type ContentHash = string;

export type FuncRunState =
  | "Created"
  | "Dispatched"
  | "Running"
  | "Postprocessing"
  | "Failure"
  | "Success";

export enum FuncKind {
  Action = "Action",
  Attribute = "Attribute",
  Authentication = "Authentication",
  CodeGeneration = "CodeGeneration",
  Intrinsic = "Intrinsic",
  Qualification = "Qualification",
  SchemaVariantDefinition = "SchemaVariantDefinition",
  Unknown = "Unknown",
  Management = "Management",
}

export enum FuncBackendKind {
  Array,
  Boolean,
  Diff,
  Identity,
  Integer,
  JsAction,
  JsAttribute,
  JsAuthentication,
  Json,
  JsSchemaVariantDefinition,
  JsValidation,
  Map,
  Object,
  String,
  Unset,
  Validation,
  Management,
}

export enum FuncBackendResponseType {
  Action,
  Array,
  Boolean,
  CodeGeneration,
  Identity,
  Integer,
  Json,
  Map,
  Object,
  Qualification,
  SchemaVariantDefinition,
  String,
  Unset,
  Validation,
  Void,
  Management,
}
export interface FuncRun {
  id: FuncRunId;
  state: FuncRunState;
  actor?: string;
  componentId?: ComponentId;
  attributeValueId?: string;
  componentName?: string;
  schemaName?: string;
  actionId?: ActionId;
  actionPrototypeId?: ActionPrototypeId;
  actionKind?: ActionKind;
  actionDisplayName?: string;
  actionOriginatingChangeSetId?: ChangeSetId;
  actionResultState?: ActionResultState;
  backendKind: FuncBackendKind;
  backendResponseType: FuncBackendResponseType;
  functionName: string;
  functionDisplayName?: string;
  functionKind: FuncKind;
  functionDescription?: string;
  functionLink?: string;
  functionArgsCasAddress: ContentHash;
  functionCodeCasAddress: ContentHash;
  resultValueCasAddress?: ContentHash;
  resultUnprocessedValueCasAddress?: ContentHash;
  createdAt: string;
  updatedAt: string;
  functionArgs: unknown;
  functionCodeBase64: string;
  resultValue: unknown;
  logs?: FuncRunLog;
  unprocessedResultValue: unknown;
}
export interface OutputLine {
  stream: string;
  execution_id: string;
  level: string;
  group?: string;
  message: string;
  timestamp: string;
}

export interface FuncRunLog {
  id: FuncRunLogId;
  createdAt: string;
  updatedAt: string;
  funcRunID: FuncRunId;
  logs: OutputLine[];
  finalized: boolean;
}

export function funcRunStatus(
  funcRun?: FuncRun,
): FuncRunState | "ActionFailure" | undefined | null {
  if (!funcRun) return null;
  if (!funcRun.state) return null;

  // If the qualification ran successfully, but it resulted in failure, then the state is a failure state.
  if (
    funcRun.functionKind === "Qualification" &&
    funcRun.state === "Success" &&
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (funcRun.unprocessedResultValue as any)?.result !== "success"
  ) {
    return "Failure";
  }

  // If actionResultState is Failure, it's an error even though state may be Success
  if (funcRun.actionResultState === "Failure") return "ActionFailure";
  return funcRun.state;
}

// move all the above types out of here for cleanliness
// leave the types below, this is the API definition!

// the route & interface definitions
// follow the pattern to make it easier on the humans!

export type GetFuncRunsPaginatedResponse = {
  funcRuns: FuncRun[];
  nextCursor: string | null;
};
export type FuncRunResponse = { funcRun: FuncRun };

export type FuncRunLogsResponse = { logs: FuncRunLog };
