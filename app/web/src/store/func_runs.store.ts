import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ActionId,
  ActionKind,
  ActionPrototypeId,
  ActionResultState,
} from "@/api/sdf/dal/action";
import { useWorkspacesStore } from "./workspaces.store";
import { AttributeValueId } from "./status.store";
import { useChangeSetsStore } from "./change_sets.store";
import handleStoreError from "./errors";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useFeatureFlagsStore } from "./feature_flags.store";

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
  Management = "management",
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

export interface ManagementHistoryItem {
  funcRunId: FuncRunId;
  name: string;
  funcId: string;
  originatingChangeSetName: string;
  updatedAt: string;
  resourceResult?: string;
  codeExecuted?: string;
  logs?: string;
  arguments?: string;
  componentName: string;
  schemaName: string;
  status: ActionResultState;
}

export interface GetFuncRunResponse {
  funcRun?: FuncRun;
}

export const useFuncRunsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const featureFlagsStore = useFeatureFlagsStore();

  const API_PREFIX = `v2/workspaces/${workspaceId}/change-sets/${changeSetId}`;

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/func_runs`, {
      state: () => ({
        funcRuns: {} as Record<FuncRunId, FuncRun>,
        lastRuns: {} as Record<ActionId, Date>,
        managementRunByPrototypeAndComponentId: {} as {
          [key: string]: FuncRunId;
        },
        managementRunHistory: {} as { [key: string]: ManagementHistoryItem[] },
      }),
      getters: {
        latestManagementRun:
          (state) => (prototypeId: string, componentId: ComponentId) =>
            state.managementRunByPrototypeAndComponentId[
              `${changeSetId ?? "NONE"}-${prototypeId}-${componentId}`
            ],
      },
      actions: {
        async GET_MANAGEMENT_RUN_HISTORY() {
          if (!featureFlagsStore.MANAGEMENT_FUNCTIONS) return;
          return new ApiRequest<ManagementHistoryItem[]>({
            url: `${API_PREFIX}/management/history`,
            headers: { accept: "application/json" },
            params: {
              visibility_change_set_pk: changeSetId,
            },
            onSuccess: (response) => {
              this.managementRunHistory[changeSetId ?? "NONE"] = response;
            },
          });
        },

        async GET_LATEST_FOR_MGMT_PROTO_AND_COMPONENT(
          prototypeId: string,
          componentId: ComponentId,
        ) {
          return new ApiRequest<FuncRun | null>({
            url: `${API_PREFIX}/management/prototype/${prototypeId}/${componentId}/latest`,
            headers: { accept: "application/json" },
            params: {
              visibility_change_set_pk: changeSetId,
            },
            onSuccess: (funcRun) => {
              if (funcRun) {
                this.setLatestManagementRun(
                  prototypeId,
                  componentId,
                  funcRun.id,
                );
                this.funcRuns[funcRun.id] = funcRun;
              }
            },
          });
        },
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

        setLatestManagementRun(
          prototypeId: string,
          componentId: string,
          funcRunId: string,
        ) {
          this.managementRunByPrototypeAndComponentId[
            `${changeSetId ?? "NONE"}-${prototypeId}-${componentId}`
          ] = funcRunId;
        },
      },
      onActivated() {
        const actionUnsub = this.$onAction(handleStoreError);
        const realtimeStore = useRealtimeStore();

        this.GET_MANAGEMENT_RUN_HISTORY();

        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "FuncRunLogUpdated",
            callback: (payload) => {
              if (payload.actionId)
                this.lastRuns[payload.actionId] = new Date();
            },
          },
          {
            eventType: "ManagementFuncExecuted",
            callback: (payload) => {
              this.setLatestManagementRun(
                payload.prototypeId,
                payload.managerComponentId,
                payload.funcRunId,
              );

              this.GET_MANAGEMENT_RUN_HISTORY();
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
