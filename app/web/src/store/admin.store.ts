import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { FuncRunId } from "@/store/func_runs.store";

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

export interface AdminUser {
  id: string;
  name: string;
  email: string;
}

export const useAdminStore = () => {
  const API_PREFIX = `v2/admin`;

  return addStoreHooks(
    null,
    null,
    defineStore(`wsNONE/admin`, {
      state: () => ({}),
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
        async LIST_WORKSPACE_USERS(workspaceId: string) {
          return new ApiRequest<{ users: AdminUser[] }>({
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/users`,
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
      },
      onActivated() {
        return () => {};
      },
    }),
  )();
};
