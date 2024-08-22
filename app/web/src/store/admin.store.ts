import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { FuncRunId } from "@/store/func_runs.store";
import { useWorkspacesStore } from "./workspaces.store";

export const useAdminStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const API_PREFIX = `v2/workspaces/${workspaceId}/admin`;

  return addStoreHooks(
    workspaceId,
    null,
    defineStore(`ws${workspaceId || "NONE"}/admin`, {
      state: () => ({}),
      actions: {
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
