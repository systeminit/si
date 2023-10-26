import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { Workspace } from "@/api/sdf/dal/workspace";
import { useAuthStore } from "./auth.store";
import { useRouterStore } from "./router.store";

type WorkspacePk = string;

type WorkspaceExportId = string;
type WorkspaceExportSummary = {
  id: WorkspaceExportId;
  createdAt: IsoDateString;
};

export const useWorkspacesStore = addStoreHooks(
  defineStore("workspaces", {
    state: () => ({
      workspacesByPk: {} as Record<WorkspacePk, Workspace>,
      workspaceExports: [] as WorkspaceExportSummary[],
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
          workspaces: Workspace[];
        }>({
          // TODO: probably should fetch list of all workspaces here...
          // something like `/users/USER_PK/workspaces`, `/my/workspaces`, etc
          url: "/session/load_workspaces",
          onSuccess: (response) => {
            this.workspacesByPk = _.keyBy(response.workspaces, "pk");

            // NOTE - we could cache this stuff in localstorage too to avoid showing loading state
            // but this is a small optimization to make later...
          },
        });
      },
      async INVITE_USER(email: string) {
        return new ApiRequest<void>({
          method: "post",
          url: "workspace/invite",
          params: {
            email,
          },
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
