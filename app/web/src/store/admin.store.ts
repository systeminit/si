import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { FuncRunId } from "@/store/func_runs.store";
import { WorkspaceUser } from "./auth.store";

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

  return addStoreHooks(
    null,
    null,
    defineStore(`wsNONE/admin`, {
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
          return new ApiRequest<{ new_modules: object[] }>({
            method: "post",
            url: `${API_PREFIX}/update_module_cache`,
          });
        },
      },
      onActivated() {
        return () => {};
      },
    }),
  )();
};
