import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ActionId,
  ActionKind,
  ActionPrototypeId,
  ActionResultState,
} from "./actions.store";
import { useWorkspacesStore } from "./workspaces.store";
import { AttributeValueId } from "./status.store";
import { useChangeSetsStore } from "./change_sets.store";

export type FuncRunId = string;
export type FuncRunLogId = string;
export type ContentHash = string;

export enum FuncRunState {
  Created = "created",
  Dispatched = "dispatched",
  Running = "running",
  PostProcessing = "postprocessing",
  Failure = "failure",
}

export enum FuncKind {
  Action = "action",
  Attribute = "attribute",
  Authentication = "authentication",
  CodeGeneration = "codeGeneration",
  Intrinsic = "intrinsic",
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
  JsReconciliation,
  JsSchemaVariantDefinition,
  JsValidation,
  Map,
  Object,
  String,
  Unset,
  Validation,
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
  Reconciliation,
  SchemaVariantDefinition,
  String,
  Unset,
  Validation,
  Void,
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

  return addStoreHooks(
    defineStore(`ws${workspaceId || "NONE"}/func_runs`, {
      state: () => ({
        funcRuns: {} as Record<FuncRunId, FuncRun>,
      }),
      actions: {
        async GET_FUNC_RUN(funcRunId: FuncRunId) {
          // note: this lookup is not cached, always re-fetch, even though the payload is large. things may have changed since last load!
          return new ApiRequest<GetFuncRunResponse>({
            url: "/func/get_func_run",
            headers: { accept: "application/json" },
            params: {
              funcRunId,
              visibility_change_set_pk: changeSetId,
            },
            onSuccess: (response) => {
              if (response.funcRun) {
                this.funcRuns[response.funcRun.id] = response.funcRun;
              }
            },
          });
        },
      },
    }),
  )();
};
