import { defineStore } from "pinia";
import _ from "lodash";
import { watch } from "vue";
import { addStoreHooks, ApiRequest } from "@si/vue-lib";
import { Workspace } from "@/api/sdf/dal/workspace";
import { useAuthStore } from "./auth.store";
import { useRouterStore } from "./router.store";

type WorkspacePk = string;

export const useWorkspacesStore = addStoreHooks(
  defineStore("workspaces", {
    state: () => ({
      workspacesByPk: {} as Record<WorkspacePk, Workspace>,
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
          url: "/session/get_defaults",
          onSuccess: (response) => {
            // this.workspacesByPk = _.keyBy(response.workspaces, "pk");
            this.workspacesByPk = _.keyBy([response.workspace], "pk");

            // NOTE - we could cache this stuff in localstorage too to avoid showing loading state
            // but this is a small optimization to make later...
          },
        });
      },
    },
    onActivated() {
      const authStore = useAuthStore();
      watch(
        () => authStore.userIsLoggedIn,
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
