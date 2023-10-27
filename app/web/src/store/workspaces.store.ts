import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";
import { addStoreHooks } from "@si/vue-lib/pinia";
import storage from "local-storage-fallback";
import { useAuthStore } from "./auth.store";
import { useRouterStore } from "./router.store";
import { AuthApiRequest } from ".";

type WorkspacePk = string;

type WorkspaceExportId = string;
type WorkspaceExportSummary = {
  id: WorkspaceExportId;
  createdAt: IsoDateString;
};

type AuthApiWorkspace = {
  creatorUserId: string;
  displayName: string;
  id: WorkspacePk;
  pk: WorkspacePk; // not actually in the response, but we backfill
  // instanceEnvType: "LOCAL" // not used yet...
  instanceUrl: string;
  role: "OWNER" | "EDITOR";
};

const LOCAL_STORAGE_LAST_WORKSPACE_PK = "si-last-workspace-pk";

export const useWorkspacesStore = addStoreHooks(
  defineStore("workspaces", {
    state: () => ({
      workspacesByPk: {} as Record<WorkspacePk, AuthApiWorkspace>,
      workspaceExports: [] as WorkspaceExportSummary[],
    }),
    getters: {
      allWorkspaces: (state) => _.values(state.workspacesByPk),
      selectedWorkspacePk(): WorkspacePk | null {
        const pk = this.selectedWorkspace?.pk || null;
        if (pk) storage.setItem(LOCAL_STORAGE_LAST_WORKSPACE_PK, pk);
        return pk;
      },
      urlSelectedWorkspaceId: () => {
        const route = useRouterStore().currentRoute;
        return route?.params?.workspacePk as WorkspacePk | undefined;
      },
      selectedWorkspace(): AuthApiWorkspace | null {
        return _.get(
          this.workspacesByPk,
          this.urlSelectedWorkspaceId || "",
          null,
        );
      },
    },

    actions: {
      getLastSelectedWorkspacePk() {
        return storage.getItem(LOCAL_STORAGE_LAST_WORKSPACE_PK) || undefined;
      },

      async FETCH_USER_WORKSPACES() {
        return new AuthApiRequest<AuthApiWorkspace[]>({
          url: "/workspaces",
          onSuccess: (response) => {
            const renameIdList = _.map(response, (w) => ({
              ...w,
              pk: w.id,
            }));
            this.workspacesByPk = _.keyBy(renameIdList, "pk");

            // NOTE - we could cache this stuff in localstorage too to avoid showing loading state
            // but this is a small optimization to make later...
          },
        });
      },
      async INVITE_USER(email: string) {
        return new AuthApiRequest<void>({
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
