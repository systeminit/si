import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import { ActionId, ActionKind, ActionPrototypeId, ActionResultState } from "@/api/sdf/dal/action";
import { useWorkspacesStore } from "./workspaces.store";
import { AttributeValueId } from "./status.store";
import { useChangeSetsStore } from "./change_sets.store";
import handleStoreError from "./errors";
import { useRealtimeStore } from "./realtime/realtime.store";

export type FuncRunId = string;
export type FuncRunLogId = string;
export type ContentHash = string;

export type FuncRunState = "Created" | "Dispatched" | "Running" | "Postprocessing" | "Failure" | "Success";

export type FuncKind = "action" | "attribute" | "authentication" | "codeGeneration" | "intrinsic" | "management";

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

// Get the Status (for StatusIndicatorIcon) from the FuncRunState
export function funcRunStatus(
  funcRun?: Pick<FuncRun, "state" | "actionResultState"> | null,
): FuncRunState | "ActionFailure" | undefined | null {
  if (!funcRun) return null;
  if (!funcRun.state) return null;

  // If actionResultState is Failure, it's an error even though state may be Success
  if (funcRun.actionResultState === "Failure") return "ActionFailure";
  return funcRun.state;
}

export interface FuncRun {
  id: FuncRunId;
  state: FuncRunState;
  actor?: string;
  componentId?: ComponentId;
  attributeValueId?: AttributeValueId;
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
}

export interface GetFuncRunResponse {
  funcRun?: FuncRun;
}

export const useFuncRunsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const realtimeStore = useRealtimeStore();

  const API_PREFIX = `v2/workspaces/${workspaceId}/change-sets/${changeSetId}`;

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/func_runs`, {
      state: () => ({
        funcRuns: {} as Record<FuncRunId, FuncRun>,
        lastRuns: {} as Record<ActionId, Date>,
      }),
      actions: {
        async GET_FUNC_RUN(funcRunId: FuncRunId) {
          // note: this lookup is not cached, always re-fetch, even though the payload is large. things may have changed since last load!
          return new ApiRequest<GetFuncRunResponse>({
            url: `${API_PREFIX}/funcs/runs/${funcRunId}`,
            headers: { accept: "application/json" },
            onSuccess: (response) => {
              if (response.funcRun) {
                this.funcRuns[response.funcRun.id] = response.funcRun;
              }
            },
          });
        },

        registerRequestsBegin(requestUlid: string, actionName: string) {
          realtimeStore.inflightRequests.set(requestUlid, actionName);
        },
        registerRequestsEnd(requestUlid: string) {
          realtimeStore.inflightRequests.delete(requestUlid);
        },
      },
      onActivated() {
        const actionUnsub = this.$onAction(handleStoreError);

        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "FuncRunLogUpdated",
            callback: (payload) => {
              // If the func run already exists, update it
              if (this.funcRuns[payload.funcRunId]) {
                this.GET_FUNC_RUN(payload.funcRunId);
              }
              if (payload.actionId) {
                this.lastRuns[payload.actionId] = new Date();
              }
            },
          },
        ]);
        return () => {
          actionUnsub();
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
