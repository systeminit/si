import { defineStore } from "pinia";
import _ from "lodash";
import { watch } from "vue";
import { ApiRequest } from "@/utils/pinia_api_tools";

import { Workspace } from "@/api/sdf/dal/workspace";
import { Organization } from "@/api/sdf/dal/organization";
import { workspace$ } from "@/observable/workspace";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { useRouterStore } from "./router.store";
import { useAuthStore } from "./auth.store";

type WorkspaceId = number;
type OrganizationId = number;

export const useWorkspacesStore = addStoreHooks(
  defineStore("workspaces", {
    state: () => ({
      workspacesById: {} as Record<WorkspaceId, Workspace>,
      organizationsById: {} as Record<OrganizationId, Organization>,
    }),
    getters: {
      allWorkspaces: (state) => _.values(state.workspacesById),
      allOrganizations: (state) => _.values(state.organizationsById),
      selectedWorkspaceId(): WorkspaceId | null {
        return this.selectedWorkspace?.id || null;
      },
      selectedWorkspace: (state) => {
        const routerStore = useRouterStore();
        const urlSelectedWorkspaceId = routerStore.urlSelectedWorkspaceId;
        return urlSelectedWorkspaceId
          ? state.workspacesById[urlSelectedWorkspaceId as WorkspaceId] || null
          : null;
      },
      // only have one org for now...
      selectedOrganization: (state) => _.values(state.organizationsById)[0],
    },
    actions: {
      async FETCH_USER_WORKSPACES() {
        return new ApiRequest<{
          workspace: Workspace;
          organization: Organization;
        }>({
          method: "get",
          // TODO: probably should fetch list of all workspaces here...
          // something like `/users/USER_ID/workspaces`, `/my/workspaces`, etc
          url: "/session/get_defaults",
          onSuccess: (response) => {
            // this.workspacesById = _.keyBy(response.workspaces, "id");
            this.workspacesById = _.keyBy([response.workspace], "id");
            this.organizationsById = _.keyBy([response.organization], "id");

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

      watch(
        () => this.selectedWorkspace,
        () => {
          workspace$.next(this.selectedWorkspace);
        },
      );
      // TODO: subscribe to realtime - changes to workspaces, or new workspaces available

      // NOTE - dont need to clean up here, since there is only one workspace store and it will always be loaded
    },
  }),
);
