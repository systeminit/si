import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { Workspace } from "@/api/sdf/dal/workspace";
import { useAuthStore } from "./auth.store";
import { useRouterStore } from "./router.store";
import { ModuleIndexApiRequest } from ".";

type WorkspacePk = string;

type WorkspaceBackupId = string;
type WorkspaceBackupSummary = {
  id: WorkspaceBackupId;
  createdAt: IsoDateString;
  metadata: {
    workspacePk: WorkspacePk;
    workspaceName: string;
  };
};

export const useWorkspacesStore = addStoreHooks(
  defineStore("workspaces", {
    state: () => ({
      workspacesByPk: {} as Record<WorkspacePk, Workspace>,
      workspaceBackups: [] as WorkspaceBackupSummary[],
    }),
    getters: {
      allWorkspaces: (state) => _.values(state.workspacesByPk),
      selectedWorkspacePk(): WorkspacePk | null {
        return this.selectedWorkspace?.pk || null;
      },
      urlSelectedWorkspaceId: () => {
        const route = useRouterStore().currentRoute;
        return route?.params?.workspacePk as WorkspacePk | undefined;
      },
      selectedWorkspace(): Workspace | null {
        return _.get(
          this.workspacesByPk,
          this.urlSelectedWorkspaceId || "",
          null,
        );
      },
    },
    actions: {
      async FETCH_USER_WORKSPACES() {
        return new ApiRequest<{
          workspace: Workspace;
        }>({
          // TODO: probably should fetch list of all workspaces here...
          // something like `/users/USER_PK/workspaces`, `/my/workspaces`, etc
          url: "/session/load_workspace",
          onSuccess: (response) => {
            // this.workspacesByPk = _.keyBy(response.workspaces, "pk");
            this.workspacesByPk = _.keyBy([response.workspace], "pk");

            // NOTE - we could cache this stuff in localstorage too to avoid showing loading state
            // but this is a small optimization to make later...
          },
        });
      },

      async EXPORT_WORKSPACE_BACKUP() {
        return new ApiRequest({
          method: "post",
          url: "/pkg/export_workspace_backup",
          params: {},
          onSuccess: (response) => {},
        });
      },
      async FETCH_WORKSPACE_BACKUPS() {
        return new ModuleIndexApiRequest<{ modules: WorkspaceBackupSummary[] }>(
          {
            url: "/backups",
            onSuccess: (response) => {
              this.workspaceBackups = response.modules;
            },
          },
        );
      },
      async RESTORE_WORKSPACE_BACKUP(backupId: WorkspaceBackupId) {
        return new ApiRequest({
          method: "post",
          url: "/pkg/restore_workspace_backup",
          params: { id: backupId },
          onSuccess: (response) => {},
        });
      },
    },
    onActivated() {
      const authStore = useAuthStore();
      watch(
        () => authStore.userIsLoggedInAndInitialized,
        (loggedIn) => {
          if (loggedIn) this.FETCH_USER_WORKSPACES();
        },
        { immediate: true },
      );

      // TODO: subscribe to realtime - changes to workspaces, or new workspaces available

      // NOTE - dont need to clean up here, since there is only one workspace store and it will always be loaded
    },
  }),
);
