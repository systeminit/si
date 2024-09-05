import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { FuncRunId } from "@/store/func_runs.store";

export interface AdminWorkspace {
  id: string;
  name: string;
  defaultChangeSetId: string;
  componentConcurrencyLimit: number;
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

interface AdminState {
  workspaces: { [key: string]: AdminWorkspace };
  changeSetsByWorkspaceId: { [key: string]: { [key: string]: AdminChangeSet } };
}

export const useAdminStore = () => {
  const API_PREFIX = `v2/admin`;

  return addStoreHooks(
    null,
    null,
    defineStore(`wsNONE/admin`, {
      state: (): AdminState => ({
        workspaces: {},
        changeSetsByWorkspaceId: {},
      }),
      actions: {
        async LIST_WORKSPACES() {
          return new ApiRequest<{
            workspaces: { [key: string]: AdminWorkspace };
          }>({
            method: "get",
            url: `${API_PREFIX}/workspaces`,
            onSuccess: (response) => {
              this.workspaces = response.workspaces;
            },
          });
        },
        async LIST_CHANGE_SETS(workspaceId: string) {
          return new ApiRequest<{
            changeSets: { [key: string]: AdminChangeSet };
          }>({
            keyRequestStatusBy: workspaceId,
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets`,
            onSuccess: (response) => {
              this.changeSetsByWorkspaceId[workspaceId] = response.changeSets;
            },
          });
        },
        async FETCH_SNAPSHOT(workspaceId: string, changeSetId: string) {
          return new ApiRequest<string>({
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/get_snapshot`,
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
        this.LIST_WORKSPACES();
        return () => {};
      },
    }),
  )();
};
