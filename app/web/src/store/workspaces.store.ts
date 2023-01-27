import { defineStore } from "pinia";
import _ from "lodash";
import { watch } from "vue";
import { ApiRequest } from "@/store/lib/pinia_api_tools";

import { Workspace } from "@/api/sdf/dal/workspace";
import { Organization } from "@/api/sdf/dal/organization";
import { addStoreHooks } from "@/store/lib/pinia_hooks_plugin";
import { useRouterStore } from "./router.store";
import { useAuthStore } from "./auth.store";

type WorkspacePk = string;
type OrganizationPk = string;

export const useWorkspacesStore = addStoreHooks(
  defineStore("workspaces", {
    state: () => ({
      workspacesByPk: {} as Record<WorkspacePk, Workspace>,
      organizationsByPk: {} as Record<OrganizationPk, Organization>,
    }),
    getters: {
      allWorkspaces: (state) => _.values(state.workspacesByPk),
      allOrganizations: (state) => _.values(state.organizationsByPk),
      selectedWorkspacePk(): WorkspacePk | null {
        return this.selectedWorkspace?.pk || null;
      },
      selectedWorkspace: (state) => {
        const routerStore = useRouterStore();
        const urlSelectedWorkspacePk = routerStore.urlSelectedWorkspacePk;
        return urlSelectedWorkspacePk
          ? state.workspacesByPk[urlSelectedWorkspacePk as WorkspacePk] || null
          : null;
      },
      // only have one org for now...
      selectedOrganization: (state) => _.values(state.organizationsByPk)[0],
    },
    actions: {
      async FETCH_USER_WORKSPACES() {
        return new ApiRequest<{
          workspace: Workspace;
          organization: Organization;
        }>({
          // TODO: probably should fetch list of all workspaces here...
          // something like `/users/USER_ID/workspaces`, `/my/workspaces`, etc
          url: "/session/get_defaults",
          onSuccess: (response) => {
            // this.workspacesByPk = _.keyBy(response.workspaces, "pk");
            this.workspacesByPk = _.keyBy([response.workspace], "pk");
            this.organizationsByPk = _.keyBy([response.organization], "pk");

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
