import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { ComponentId } from "@/api/sdf/dal/component";
import { ActionResultState } from "@/api/sdf/dal/action";
import { useWorkspacesStore } from "./workspaces.store";
import { useChangeSetsStore } from "./change_sets.store";
import handleStoreError from "./errors";
import { useRealtimeStore } from "./realtime/realtime.store";
import { FuncRun, FuncRunId, FuncRunState, useFuncRunsStore } from "./func_runs.store";

export interface ManagementHistoryItem {
  id: FuncRunId;
  state: FuncRunState;
  functionName: string;
  functionDisplayName?: string;
  componentId: string;
  componentName: string;
  schemaName: string;
  originatingChangeSetName: string;
  updatedAt: string;
  actionResultState?: ActionResultState;
}

export const useManagementRunsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const funcRunsStore = useFuncRunsStore();
  const realtimeStore = useRealtimeStore();

  const API_PREFIX = `v2/workspaces/${workspaceId}/change-sets/${changeSetId}`;

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId}/management_runs`, {
      state: () => ({
        managementRunByPrototypeAndComponentId: {} as {
          [key: string]: FuncRunId;
        },
        managementRunHistory: [] as ManagementHistoryItem[],
      }),
      getters: {
        latestManagementRun: (state) => (prototypeId: string, componentId: ComponentId) =>
          state.managementRunByPrototypeAndComponentId[`${prototypeId}-${componentId}`],
      },
      actions: {
        async GET_MANAGEMENT_RUN_HISTORY() {
          return new ApiRequest<ManagementHistoryItem[]>({
            url: `${API_PREFIX}/management/history`,
            headers: { accept: "application/json" },
            params: {
              visibility_change_set_pk: changeSetsStore.selectedChangeSetId,
            },
            onSuccess: (response) => {
              this.managementRunHistory = response;
            },
          });
        },

        async GET_LATEST_FOR_MGMT_PROTO_AND_COMPONENT(prototypeId: string, componentId: ComponentId) {
          return new ApiRequest<FuncRun | null>({
            url: `${API_PREFIX}/management/prototype/${prototypeId}/${componentId}/latest`,
            headers: { accept: "application/json" },
            params: {
              visibility_change_set_pk: changeSetsStore.selectedChangeSetId,
            },
            onSuccess: (funcRun) => {
              if (funcRun) {
                this.setLatestManagementRun(prototypeId, componentId, funcRun.id);
                funcRunsStore.funcRuns[funcRun.id] = funcRun;
              }
            },
          });
        },

        setLatestManagementRun(prototypeId: string, componentId: string, funcRunId: string) {
          this.managementRunByPrototypeAndComponentId[`${prototypeId}-${componentId}`] = funcRunId;
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

        this.GET_MANAGEMENT_RUN_HISTORY();

        const changeSetId = changeSetsStore.selectedChangeSetId;
        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "ManagementFuncExecuted",
            callback: (payload) => {
              this.setLatestManagementRun(payload.prototypeId, payload.managerComponentId, payload.funcRunId);

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
