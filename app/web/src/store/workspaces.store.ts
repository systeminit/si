import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { watch } from "vue";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import storage from "local-storage-fallback";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import router from "@/router";
import { ModuleId } from "@/api/sdf/dal/module";
import { WorkspacePk } from "@/api/sdf/dal/workspace";
import { useAuthStore, UserId } from "./auth.store";
import { useRouterStore } from "./router.store";
import handleStoreError from "./errors";
import { AuthApiRequest } from ".";

export type WorkspaceIntegrationId = string;

type WorkspaceExportId = string;
type WorkspaceExportSummary = {
  id: WorkspaceExportId;
  createdAt: IsoDateString;
};

type InstanceEnvType = "LOCAL" | "PRIVATE" | "SI";

type AuthApiWorkspace = {
  creatorUserId: string;
  displayName: string;
  id: WorkspacePk;
  pk: WorkspacePk; // not actually in the response, but we backfill
  instanceEnvType: InstanceEnvType;
  instanceUrl: string;
  role: "OWNER" | "EDITOR";
  token: string;
  isHidden: boolean;
  approvalsEnabled: boolean;
};

export type WorkspaceImportSummary = {
  importRequestedByUserPk: UserId;
  workspaceExportCreatedAt: IsoDateString;
  workspaceExportCreatedBy: string;
  importedWorkspaceName: string;
};

export type WorkspaceIntegration = {
  pk: WorkspaceIntegrationId;
  workspaceId: WorkspacePk;
  slackWebhookUrl?: string;
};

const LOCAL_STORAGE_LAST_WORKSPACE_PK = "si-last-workspace-pk";

// Note(victor): The workspace import exists outside a change set context
// (since change sets exists inside tenancies) - So no endpoints in this store
// should use a visibility. If one seems like it should, then it belongs
// in a different store.
export const useWorkspacesStore = () => {
  const realtimeStore = useRealtimeStore();

  return addStoreHooks(
    undefined,
    undefined,
    defineStore("workspaces", {
      state: () => ({
        workspacesByPk: {} as Record<WorkspacePk, AuthApiWorkspace>,
        workspaceExports: [] as WorkspaceExportSummary[],
        workspaceImportSummary: null as WorkspaceImportSummary | null,
        workspaceApprovals: {} as Record<UserId, string>,
        importCompletedAt: null as IsoDateString | null,
        importCancelledAt: null as IsoDateString | null,
        importId: null as string | null,
        importLoading: false as boolean,
        importError: undefined as string | undefined,
        integrations: null as WorkspaceIntegration | null,
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
        workspaceApprovalsEnabled(): boolean {
          const thisWorkspace = this.selectedWorkspace || null;
          if (!thisWorkspace) return false;
          return thisWorkspace.approvalsEnabled;
        },
        getIntegrations(): WorkspaceIntegration | null {
          return this.integrations || null;
        },
      },

      actions: {
        getAutoSelectedWorkspacePk() {
          const lastSelected = storage.getItem(LOCAL_STORAGE_LAST_WORKSPACE_PK);
          // here we can inject extra logic for auto selection...
          return lastSelected || this.allWorkspaces[0]?.pk;
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
        async BEGIN_WORKSPACE_IMPORT(moduleId: ModuleId) {
          this.workspaceApprovals = {};
          this.importId = null;
          this.importLoading = true;
          this.importError = undefined;
          return new ApiRequest<{ id: string }>({
            method: "post",
            url: `/v2/workspaces/${moduleId}/install`,
            onSuccess: (data) => {
              this.workspaceImportSummary = null;
              this.importId = data.id;
            },
            onFail: () => {
              this.importId = null;
              this.importLoading = false;
            },
          });
        },
        async BEGIN_APPROVAL_PROCESS(moduleId: ModuleId) {
          return new ApiRequest({
            method: "post",
            url: "/module/begin_approval_process",
            params: {
              id: moduleId,
            },
          });
        },
        async CANCEL_APPROVAL_PROCESS() {
          this.workspaceImportSummary = null;
          return new ApiRequest({
            method: "post",
            url: "/module/cancel_approval_process",
            params: {},
            onSuccess: (_response) => {
              this.workspaceImportSummary = null;
            },
          });
        },
        async IMPORT_WORKSPACE_VOTE(vote: string) {
          return new ApiRequest({
            method: "post",
            url: "/module/import_workspace_vote",
            params: {
              vote,
            },
          });
        },
        async GET_INTEGRATIONS() {
          if (
            this.selectedWorkspacePk === null ||
            this.selectedWorkspacePk === ""
          )
            return;
          return new ApiRequest({
            method: "get",
            url: `v2/workspaces/${this.selectedWorkspacePk}/integrations`,
            onSuccess: (response) => {
              if (response)
                this.integrations = {
                  pk: response.integration.pk,
                  workspaceId: response.integration.workspace_pk,
                  slackWebhookUrl: response.integration.slack_webhook_url,
                };
            },
          });
        },
        async UPDATE_INTEGRATION(
          workspaceIntegrationId: WorkspaceIntegrationId,
          webhookUrl: string,
        ) {
          if (
            this.selectedWorkspacePk === null ||
            this.selectedWorkspacePk === ""
          )
            return;
          return new ApiRequest({
            method: "post",
            url: `v2/workspaces/${this.selectedWorkspacePk}/integrations/${workspaceIntegrationId}`,
            params: {
              slackWebhookUrl: webhookUrl,
            },
            onSuccess: (response) => {
              this.integrations = response.integration;
            },
          });
        },

        registerRequestsBegin(requestUlid: string, actionName: string) {
          realtimeStore.inflightRequests.set(requestUlid, actionName);
        },
        registerRequestsEnd(requestUlid: string) {
          realtimeStore.inflightRequests.delete(requestUlid);
        },
      },

      onActivated() {
        const authStore = useAuthStore();
        watch(
          () => authStore.userIsLoggedInAndInitialized,
          (loggedIn) => {
            if (loggedIn) {
              this.FETCH_USER_WORKSPACES();
            }
          },
          { immediate: true },
        );

        // Since there is only one workspace store instance,
        // we need to resubscribe when the workspace pk changes
        watch(
          () => this.selectedWorkspacePk,
          () => {
            this.GET_INTEGRATIONS();
            realtimeStore.subscribe(
              this.$id,
              `workspace/${this.selectedWorkspacePk}`,
              [
                {
                  eventType: "WorkspaceImportBeginApprovalProcess",
                  callback: (data) => {
                    this.importCancelledAt = null;
                    this.importCompletedAt = null;
                    this.workspaceImportSummary = {
                      importRequestedByUserPk: data.userPk,
                      workspaceExportCreatedAt: data.createdAt,
                      workspaceExportCreatedBy: data.createdBy,
                      importedWorkspaceName: data.name,
                    };
                  },
                },
                {
                  eventType: "WorkspaceImportCancelApprovalProcess",
                  callback: () => {
                    this.workspaceApprovals = {};
                    this.workspaceImportSummary = null;
                    this.importCancelledAt = new Date().toISOString();
                    this.importCompletedAt = null;
                  },
                },
                {
                  eventType: "ImportWorkspaceVote",
                  callback: (data) => {
                    if (this.selectedWorkspacePk === data.workspacePk) {
                      this.workspaceApprovals[data.userPk] = data.vote;
                    }
                  },
                },
                {
                  eventType: "WorkspaceImported",
                  callback: () => {
                    this.workspaceApprovals = {};
                    this.workspaceImportSummary = null;
                    this.importCompletedAt = new Date().toISOString();
                    this.importCancelledAt = null;
                  },
                },
                {
                  eventType: "AsyncFinish",
                  callback: ({ id }: { id: string }) => {
                    if (id === this.importId) {
                      this.importLoading = false;
                      this.importCompletedAt = new Date().toISOString();
                      this.importError = undefined;
                      this.importId = null;

                      const route = router.currentRoute.value;

                      router.push({
                        name: "workspace-compose",
                        params: {
                          workspacePk: route.params.workspacePk,
                          changeSetId: "head",
                        },
                      });
                    }
                  },
                },
                {
                  eventType: "AsyncError",
                  callback: ({ id, error }: { id: string; error: string }) => {
                    if (id === this.importId) {
                      this.importLoading = false;
                      this.importError = error;
                      this.importId = null;
                    }
                  },
                },
              ],
            );
          },
          { immediate: true },
        );

        this.$onAction(handleStoreError);

        // NOTE - don't need to clean up here, since there is only one workspace
        // store, and it will always be loaded
      },
    }),
  )();
};
