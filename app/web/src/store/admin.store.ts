import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { FuncRunId } from "@/store/func_runs.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { WorkspaceUser } from "@/store/auth.store";

export interface AdminWorkspace {
  id: string;
  name: string;
  defaultChangeSetId: string;
  componentConcurrencyLimit?: number;
  snapshotVersion: string;
}

export interface AdminChangeSet {
  id: string;
  name: string;
  status: string;
  baseChangeSetId?: string;
  workspaceSnapshotAddress: string;
  workspaceId: string;
  mergeRequestedByUserId?: string;
}

export const useAdminStore = () => {
  const API_PREFIX = `v2/admin`;
  const WORKSPACE_API_PREFIX = ["v2", "workspaces"];

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const realtimeStore = useRealtimeStore();

  return addStoreHooks(
    workspaceId,
    null,
    defineStore(`ws${workspaceId || "NONE"}/admin`, {
      state: () => ({
        updatingModuleCacheOperationId: null as string | null,
        updatingModuleCacheOperationError: undefined as string | undefined,
        updatingModuleCacheOperationRunning: false as boolean,
        clearingInnitCacheOperationRunning: false as boolean,
      }),
      getters: {},
      actions: {
        async SEARCH_WORKSPACES(query?: string) {
          return new ApiRequest<
            {
              workspaces: AdminWorkspace[];
            },
            { query?: string }
          >({
            method: "get",
            params: { query },
            url: `${API_PREFIX}/workspaces`,
          });
        },
        async LIST_CHANGE_SETS(workspaceId: string) {
          return new ApiRequest<{
            changeSets: { [key: string]: AdminChangeSet };
          }>({
            keyRequestStatusBy: workspaceId,
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets`,
          });
        },
        // TODO(nick): remove in favor of the auth store call.
        async LIST_WORKSPACE_USERS(workspaceId: string) {
          return new ApiRequest<{ users: WorkspaceUser[] }>({
            method: "get",
            url: WORKSPACE_API_PREFIX.concat([workspaceId, "users"]),
          });
        },
        async GET_SNAPSHOT(workspaceId: string, changeSetId: string) {
          return new ApiRequest<string>({
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/get_snapshot`,
          });
        },
        async SET_SNAPSHOT(
          workspaceId: string,
          changeSetId: string,
          snapshot: Blob,
        ) {
          const formData = new FormData();
          formData.append("snapshot", snapshot);

          return new ApiRequest<{ workspaceSnapshotAddress: string }>({
            method: "post",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/set_snapshot`,
            formData,
          });
        },
        async GET_CAS_DATA(workspaceId: string, changeSetId: string) {
          return new ApiRequest<string>({
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/get_cas_data`,
          });
        },
        async UPLOAD_CAS_DATA(
          workspaceId: string,
          changeSetId: string,
          casData: Blob,
        ) {
          const formData = new FormData();
          formData.append("cas_data", casData);

          return new ApiRequest<{ workspaceSnapshotAddress: string }>({
            method: "post",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/upload_cas_data`,
            formData,
          });
        },
        async SET_CONCURRENCY_LIMIT(
          workspaceId: string,
          concurrencyLimit?: number,
        ) {
          return new ApiRequest<
            { concurrencyLimit?: number },
            { concurrencyLimit?: number }
          >({
            method: "post",
            url: `${API_PREFIX}/workspaces/${workspaceId}/set_concurrency_limit`,
            params: { concurrencyLimit },
          });
        },
        async KILL_EXECUTION(funcRunId: FuncRunId) {
          return new ApiRequest<null>({
            method: "put",
            url: `${API_PREFIX}/func/runs/${funcRunId}/kill_execution`,
          });
        },
        async UPDATE_MODULE_CACHE() {
          this.updatingModuleCacheOperationRunning = true;
          this.updatingModuleCacheOperationId = null;
          this.updatingModuleCacheOperationError = undefined;

          return new ApiRequest<{ id: string }>({
            method: "post",
            url: `${API_PREFIX}/update_module_cache`,
            onSuccess: (response) => {
              this.updatingModuleCacheOperationId = response.id;
            },
            onFail: () => {
              this.updatingModuleCacheOperationRunning = false;
            },
          });
        },
        async CLEAR_INNIT_CACHE() {
          this.clearingInnitCacheOperationRunning = true;

          return new ApiRequest<{ id: string }>({
            method: "post",
            url: `${API_PREFIX}/innit/cache/clear`,
            onSuccess: (response) => {
              this.clearingInnitCacheOperationRunning = false;
            },
            onFail: () => {
              this.clearingInnitCacheOperationRunning = false;
            },
          });
        },
      },
      onActivated() {
        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}`, [
          {
            eventType: "AsyncFinish",
            callback: async ({ id }: { id: string }) => {
              if (id === this.updatingModuleCacheOperationId) {
                this.updatingModuleCacheOperationRunning = false;
              }
            },
          },
          {
            eventType: "AsyncError",
            callback: async ({ id, error }: { id: string; error: string }) => {
              if (id === this.updatingModuleCacheOperationId) {
                this.updatingModuleCacheOperationError = error;
                this.updatingModuleCacheOperationRunning = false;
              }
            },
          },
        ]);
        return () => {};
      },
    }),
  )();
};
