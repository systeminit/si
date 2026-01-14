import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { useToast } from "vue-toastification";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { URLPattern } from "@si/vue-lib";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { SchemaId, SchemaVariantId } from "@/api/sdf/dal/schema";
import { Visibility } from "@/api/sdf/dal/visibility";
import { LatestModule, ModuleContributeRequest, ModuleId } from "@/api/sdf/dal/module";
import { useChangeSetsStore } from "./change_sets.store";
import { useRouterStore } from "./router.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { ModuleIndexApiRequest } from ".";

export type ModuleName = string;
export type ModuleHash = string;
export type ModuleSlug = ModuleHash;

export interface ModuleFuncView {
  name: string;
  displayName?: string;
  description?: string;
}

export interface LocalModuleSummary {
  name: string;
  hash: ModuleHash;
  isBuiltin: boolean;
}

export interface LocalModuleDetails {
  name: string;
  version: string;
  description: string;
  createdAt: IsoDateString;
  createdBy: string;
  schemas: string[];
  funcs: ModuleFuncView[];
  hash: ModuleHash;
  kind: "module" | "workspaceExport";
}

export interface ModuleSpec {
  name: string;
  createdAt: IsoDateString;
  createdBy: string;
  version: string;
  funcs: {
    arguments: {
      name: string;
      kind: string;
      elementKind: string | null;
    }[];
    data: {
      backendKind: string;
      codeBase64: string | null;
      description: string | null;
      displayName: string | null;
      handler: string | null;
      hidden: boolean;
      link: string | null;
      name: string;
      responseType: string;
    };
    name: string;
    uniqueId: string;
  }[];
}

export type Asset = {
  id: number;
  displayName: string;
};

export type RemoteModuleSummary = {
  id: ModuleId;
  name: string;
  description: string;
  createdAt: IsoDateString;
  hash: ModuleHash;
  hashCreatedAt: IsoDateString;
  ownerDisplayName: string;
  ownerUserId: string; // userid?
  isBuiltin: boolean; // only set for builtins
};

export type RemoteModuleDetails = RemoteModuleSummary & {
  metadata?: {
    schemas: string[];
    funcs: ModuleFuncView[];
    version: string;
  };
};

// Gather the current change set ID, since our components don't appear to be
// reacting to the visibility changes that happen on the setup code
// TODO: Generalize this and make the api client pass these arguments implicitly
function getVisibilityParams(forceChangeSetId?: ChangeSetId) {
  const changeSetsStore = useChangeSetsStore();
  let changeSetId = changeSetsStore.selectedChangeSetId;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  }
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  return {
    visibility_change_set_pk: changeSetId,
    workspaceId,
  };
}

export const useModuleStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const visibility = {
    // changeSetId should not be empty if we are actually using this store
    // so we can give it a bad value and let it throw an error
    visibility_change_set_pk: changeSetId || "XXX",
  };

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const routerStore = useRouterStore();
  const realtimeStore = useRealtimeStore();
  const toast = useToast();

  const API_PREFIX = ["v2", "workspaces", { workspaceId }, "change-sets", { changeSetId }, "modules"] as URLPattern;

  const WORKSPACE_API_PREFIX = ["v2", "workspaces", { workspaceId }];

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/modules`, {
      state: () => ({
        updatingModulesOperationId: null as string | null,
        updatingModulesOperationError: undefined as string | undefined,
        updatingModulesOperationRunning: false as boolean,
        upgradeableModules: {} as Record<SchemaVariantId, LatestModule>,
        contributableModules: [] as SchemaVariantId[],
        localModulesByName: {} as Record<ModuleName, LocalModuleSummary>,
        localModuleDetailsByName: {} as Record<ModuleName, LocalModuleDetails>,
        remoteModuleList: [] as RemoteModuleSummary[],
        builtinsSearchResults: [] as RemoteModuleSummary[],
        remoteModuleDetailsById: {} as Record<ModuleId, RemoteModuleDetails>,
        remoteModuleSpecsById: {} as Record<ModuleId, ModuleSpec>,

        exportingWorkspaceOperationId: null as string | null,
        exportingWorkspaceOperationError: undefined as string | undefined,
        exportingWorkspaceOperationRunning: false as boolean,
      }),
      getters: {
        urlSelectedModuleSlug: () => {
          const route = useRouterStore().currentRoute;
          return route?.params?.moduleSlug as ModuleSlug | undefined;
        },
        localModules: (state) => _.values(state.localModulesByName),
        localModulesByHash(): Record<ModuleSlug, LocalModuleSummary> {
          return _.keyBy(this.localModules, (m) => m.hash);
        },

        localModuleDetails: (state) => _.values(state.localModuleDetailsByName),
        localModuleDetailsByHash(): Record<ModuleSlug, LocalModuleDetails> {
          return _.keyBy(this.localModuleDetails, (m) => m.hash);
        },

        remoteModuleSummaryByHash: (state) => {
          return _.keyBy(state.remoteModuleList, (m) => m.hash);
        },
        remoteModuleDetailsByHash: (state) => {
          return _.keyBy(_.values(state.remoteModuleDetailsById), (m) => m.hash);
        },
        builtins: (state) => _.values(state.builtinsSearchResults),
        builtinModuleSummaryByHash: (state) => _.keyBy(state.builtinsSearchResults, (m) => m.hash),
        builtinModuleDetailsByHash: (state) => _.keyBy(state.builtinsSearchResults, (m) => m.hash),
        selectedModuleLocalSummary(): LocalModuleSummary | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.localModulesByHash[this.urlSelectedModuleSlug];
        },
        selectedModuleLocalDetails(): LocalModuleDetails | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.localModuleDetailsByHash[this.urlSelectedModuleSlug];
        },
        selectedModuleRemoteSummary(): RemoteModuleDetails | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.remoteModuleSummaryByHash[this.urlSelectedModuleSlug];
        },
        selectedModuleRemoteDetails(): RemoteModuleDetails | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.remoteModuleDetailsByHash[this.urlSelectedModuleSlug];
        },
        selectedBuiltinModuleDetails(): RemoteModuleDetails | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.builtinModuleDetailsByHash[this.urlSelectedModuleSlug];
        },
        selectedBuiltinModuleSummary(): RemoteModuleDetails | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.builtinModuleSummaryByHash[this.urlSelectedModuleSlug];
        },
      },
      actions: {
        // Modules API V2
        async SYNC() {
          return new ApiRequest<
            {
              upgradeable: Record<SchemaVariantId, LatestModule>;
              contributable: SchemaVariantId[];
            },
            Visibility
          >({
            url: API_PREFIX.concat(["sync"]),
            params: { ...visibility },
            onSuccess: (response) => {
              this.upgradeableModules = response.upgradeable;
              this.contributableModules = response.contributable;
            },
          });
        },

        async CONTRIBUTE(request: ModuleContributeRequest) {
          return new ApiRequest({
            method: "post",
            url: API_PREFIX.concat(["contribute"]),
            params: {
              name: request.name,
              version: request.version,
              schemaVariantId: request.schemaVariantId,
              isPrivateModule: request.isPrivateModule,
            },
          });
        },

        async LOAD_LOCAL_MODULES() {
          return new ApiRequest<LocalModuleSummary[]>({
            method: "get",
            url: API_PREFIX,
            onSuccess: (response) => {
              // TODO: remove this
              // the backend currently needs the full tar file name
              // but we want the actual name in the module metadata
              // easier to strip off temporarily but we'll need to change what the backend is storing
              const modulesWithNamesFixed = _.map(response, (m) => ({
                ...m,
                name: m.name.replace(/-\d\d\d\d-\d\d-\d\d\.sipkg/, ""),
              }));

              this.localModulesByName = _.keyBy(modulesWithNamesFixed, (m) => m.name);
            },
          });
        },

        async GET_LOCAL_MODULE_DETAILS(hash: ModuleHash) {
          return new ApiRequest<LocalModuleDetails>({
            method: "get",
            url: API_PREFIX.concat(["module_by_hash"]),
            params: { hash },
            onSuccess: (response) => {
              this.localModuleDetailsByName[response.name] = response;
            },
          });
        },

        async GET_REMOTE_MODULE_SPEC(id: ModuleId) {
          return new ApiRequest<ModuleSpec>({
            method: "get",
            url: API_PREFIX.concat(["module_by_id"]),
            keyRequestStatusBy: id,
            params: { id },
            onSuccess: (response) => {
              this.remoteModuleSpecsById[id] = response;
            },
          });
        },

        // Module Index API
        async LIST_WORKSPACE_EXPORTS() {
          return new ModuleIndexApiRequest<{
            modules: (RemoteModuleSummary & {
              latestHash: ModuleHash;
              latestHashCreatedAt: IsoDateString;
            })[];
          }>({
            method: "get",
            url: "/modules",
            params: { kind: "workspaceBackup" },
          });
        },

        async LIST_BUILTINS() {
          return new ModuleIndexApiRequest<{
            modules: (RemoteModuleSummary & {
              latestHash: ModuleHash;
              latestHashCreatedAt: IsoDateString;
            })[];
          }>({
            method: "get",
            url: "/builtins",
            onSuccess: (response) => {
              this.builtinsSearchResults = _.map(response.modules, (m) => ({
                ...m,
                hash: m.latestHash,
                hashCreatedAt: m.latestHashCreatedAt,
                isBuiltin: true,
              }));
            },
          });
        },

        async GET_REMOTE_MODULES_LIST(params?: { kind?: string; su?: boolean }) {
          return new ModuleIndexApiRequest<{
            modules: (RemoteModuleSummary & {
              latestHash: ModuleHash;
              latestHashCreatedAt: IsoDateString;
            })[];
          }>({
            method: "get",
            url: "/modules",
            params,
            onSuccess: (response) => {
              this.remoteModuleList = _.map(response.modules, (m) => ({
                ...m,
                hash: m.latestHash,
                hashCreatedAt: m.latestHashCreatedAt,
              }));
            },
          });
        },

        async GET_REMOTE_MODULE_DETAILS(id: ModuleId) {
          return new ModuleIndexApiRequest<RemoteModuleDetails>({
            method: "get",
            url: `/modules/${id}`,
            onSuccess: (
              response: RemoteModuleDetails & {
                latestHash: ModuleHash;
                latestHashCreatedAt: IsoDateString;
              },
            ) => {
              response.hash = response.latestHash;
              response.hashCreatedAt = response.latestHashCreatedAt;

              this.remoteModuleDetailsById[response.id] = response;
            },
          });
        },

        async INSTALL_REMOTE_MODULE(moduleIds: ModuleId[]) {
          if (changeSetsStore.creatingChangeSet) {
            throw new Error("race, wait until the change set is created");
          }
          if (changeSetId === changeSetsStore.headChangeSetId) {
            changeSetsStore.creatingChangeSet = true;
          }

          return new ApiRequest<{ id: string }>({
            method: "post",
            url: "/module/install_module",
            keyRequestStatusBy: moduleIds,
            params: {
              ids: moduleIds,
              ...getVisibilityParams(),
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
            onSuccess: () => {
              // reset installed list
              this.SYNC();
            },
          });
        },

        async UPGRADE_MODULES(schemaIds: SchemaId[]) {
          if (changeSetsStore.creatingChangeSet) {
            throw new Error("race, wait until the change set is created");
          }
          if (changeSetId === changeSetsStore.headChangeSetId) {
            changeSetsStore.creatingChangeSet = true;
          }

          this.updatingModulesOperationRunning = true;
          this.updatingModulesOperationId = null;
          this.updatingModulesOperationError = undefined;

          return new ApiRequest<string>({
            method: "post",
            url: "/module/upgrade_modules",
            keyRequestStatusBy: schemaIds,
            params: {
              schemaIds,
              ...getVisibilityParams(),
            },
            optimistic: () => {
              toast("Upgrading modules...");
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
              this.updatingModulesOperationRunning = false;
            },
            onSuccess: (response) => {
              this.updatingModulesOperationId = response;
            },
          });
        },

        async INSTALL_MODULE_FROM_FILE(file: File) {
          if (changeSetsStore.creatingChangeSet) {
            throw new Error("race, wait until the change set is created");
          }
          if (changeSetId === changeSetsStore.headChangeSetId) {
            changeSetsStore.creatingChangeSet = true;
          }

          const formData = new FormData();
          formData.append("pkg_spec", file);

          return new ApiRequest<{ id: string }>({
            method: "post",
            url: API_PREFIX.concat(["install_from_file"]),
            headers: {
              "Content-Type": "multipart/form-data",
            },
            formData,
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
            onSuccess: () => {
              // reset installed list
              this.SYNC();
            },
          });
        },

        async REJECT_REMOTE_MODULE(moduleId: ModuleId) {
          return new ApiRequest<{ success: true }>({
            method: "post",
            url: API_PREFIX.concat([{ moduleId }, "builtins", "reject"]),
            params: { id: moduleId, ...getVisibilityParams() },
            optimistic: () => {
              // remove selection from URL
              routerStore.replace(changeSetId, {
                name: "workspace-lab-packages",
              });
            },
            onSuccess: (_response) => {
              // response is just success, so we have to reload the remote modules
              this.LOAD_LOCAL_MODULES();
              this.GET_REMOTE_MODULES_LIST({ su: true });
            },
          });
        },

        async PROMOTE_TO_BUILTIN(moduleId: ModuleId) {
          return new ApiRequest<{ success: true }>({
            method: "post",
            url: API_PREFIX.concat([{ moduleId }, "builtins", "promote"]),
            optimistic: () => {
              // remove selection from URL
              routerStore.replace(changeSetId, {
                name: "workspace-lab-packages",
              });
            },
            onSuccess: (_response) => {
              // response is just success, so we have to reload the remote modules
              this.LOAD_LOCAL_MODULES();
              this.GET_REMOTE_MODULES_LIST({ su: true });
            },
          });
        },

        async EXPORT_WORKSPACE() {
          this.exportingWorkspaceOperationId = null;
          this.exportingWorkspaceOperationError = undefined;
          this.exportingWorkspaceOperationRunning = true;

          return new ApiRequest<{ id: string }>({
            method: "post",
            url: WORKSPACE_API_PREFIX.concat(["export"]),
            onSuccess: (response) => {
              this.exportingWorkspaceOperationId = response.id;
            },
            onFail: () => {
              this.exportingWorkspaceOperationRunning = false;
            },
          });
        },

        resetExportWorkspaceStatus() {
          this.exportingWorkspaceOperationRunning = false;
          this.exportingWorkspaceOperationId = null;
          this.exportingWorkspaceOperationError = undefined;
        },

        registerRequestsBegin(requestUlid: string, actionName: string) {
          realtimeStore.inflightRequests.set(requestUlid, actionName);
        },
        registerRequestsEnd(requestUlid: string) {
          realtimeStore.inflightRequests.delete(requestUlid);
        },
      },
      async onActivated() {
        // This store is activated very early, and we might not have a change
        // set yet. This guards against an errant 500 in that case.
        const visibilityParams = getVisibilityParams();
        if (visibilityParams.workspaceId && visibilityParams.visibility_change_set_pk) {
          await this.LOAD_LOCAL_MODULES();
        }

        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}`, [
          {
            eventType: "AsyncFinish",
            callback: async ({ id }: { id: string }) => {
              if (id === this.exportingWorkspaceOperationId) {
                this.exportingWorkspaceOperationRunning = false;
              }
              if (id === this.updatingModulesOperationId) {
                this.updatingModulesOperationRunning = false;
                this.SYNC();
              }
            },
          },
          {
            eventType: "ModuleImported",
            callback: (schemaVariants, metadata) => {
              if (metadata.change_set_id !== changeSetId) return;
              for (const variant of schemaVariants) {
                delete this.upgradeableModules[variant.schemaVariantId];
              }
            },
          },
          {
            eventType: "AsyncError",
            callback: async ({ id, error }: { id: string; error: string }) => {
              if (id === this.exportingWorkspaceOperationId) {
                this.exportingWorkspaceOperationError = error;
                this.exportingWorkspaceOperationRunning = false;
              }
              if (id === this.updatingModulesOperationId) {
                this.updatingModulesOperationError = error;
                this.updatingModulesOperationRunning = false;
              }
            },
          },
        ]);

        return () => {
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
