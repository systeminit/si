import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { DiagramInputSocket, DiagramOutputSocket } from "@/api/sdf/dal/diagram";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "./change_sets.store";
import { useRouterStore } from "./router.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { ModuleIndexApiRequest } from ".";

export type ModuleId = string;
export type ModuleName = string;
export type ModuleHash = string;
export type ModuleSlug = ModuleHash;

export interface SchemaVariant {
  id: string;
  name: string;
  schemaName: string;
  schemaId: string;
  color: string;
  inputSockets: DiagramInputSocket[];
  outputSockets: DiagramOutputSocket[];
}

export interface PkgFuncView {
  name: string;
  displayName?: string;
  description?: string;
}

export interface PkgExportRequest {
  name: string;
  version: string;
  description?: string;
  schemaVariants: string[];
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
  funcs: PkgFuncView[];
  hash: ModuleHash;
  kind: "module" | "workspaceExport";
}

export interface ModuleSpec {
  funcs: {
    arguments: {
      name: string;
      kind: string;
      elementKind: string | null;
    }[];
    backendKind: string;
    codeBase64: string | null;
    description: string | null;
    displayName: string | null;
    handler: string | null;
    hidden: boolean;
    link: string | null;
    name: string;
    responseType: string;
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
    funcs: PkgFuncView[];
    version: string;
  };
};

export const useModuleStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: changeSetId ?? nilId(),
  };

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/modules`,
      {
        state: () => ({
          localModulesByName: {} as Record<ModuleName, LocalModuleSummary>,
          localModuleDetailsByName: {} as Record<
            ModuleName,
            LocalModuleDetails
          >,
          remoteModuleSearchResults: [] as RemoteModuleSummary[],
          builtinsSearchResults: [] as RemoteModuleSummary[],
          remoteModuleDetailsById: {} as Record<ModuleId, RemoteModuleDetails>,
          remoteModuleSpecsById: {} as Record<ModuleId, ModuleSpec>,

          installingModuleId: null as string | null,
          installingError: undefined as string | undefined,
          installingLoading: false as boolean,

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

          localModuleDetails: (state) =>
            _.values(state.localModuleDetailsByName),
          localModuleDetailsByHash(): Record<ModuleSlug, LocalModuleDetails> {
            return _.keyBy(this.localModuleDetails, (m) => m.hash);
          },

          remoteModuleSummaryByHash: (state) => {
            return _.keyBy(state.remoteModuleSearchResults, (m) => m.hash);
          },
          remoteModuleDetailsByHash: (state) => {
            return _.keyBy(
              _.values(state.remoteModuleDetailsById),
              (m) => m.hash,
            );
          },
          builtinModuleSummaryByHash: (state) =>
            _.keyBy(state.builtinsSearchResults, (m) => m.hash),
          builtinModuleDetailsByHash: (state) =>
            _.keyBy(state.builtinsSearchResults, (m) => m.hash),
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
          async LOAD_LOCAL_MODULES() {
            return new ApiRequest<{ pkgs: LocalModuleSummary[] }>({
              url: "/pkg/list_pkgs",
              params: { ...visibility },
              onSuccess: (response) => {
                // TODO: remove this
                // the backend currently needs the full tar file name
                // but we want the actual name in the module metadata
                // easier to strip off temporarily but we'll need to change what the backend is storing
                const modulesWithNamesFixed = _.map(response.pkgs, (m) => ({
                  ...m,
                  name: m.name.replace(/-\d\d\d\d-\d\d-\d\d\.sipkg/, ""),
                }));

                this.localModulesByName = _.keyBy(
                  modulesWithNamesFixed,
                  (m) => m.name,
                );
              },
            });
          },

          async GET_LOCAL_MODULE_DETAILS(hash: ModuleHash) {
            return new ApiRequest<LocalModuleDetails>({
              method: "get",
              url: "/pkg/get_module_by_hash",
              params: { hash, ...visibility },
              onSuccess: (response) => {
                this.localModuleDetailsByName[response.name] = response;
              },
            });
          },

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

          async SEARCH_REMOTE_MODULES(params?: {
            name?: string;
            kind?: string;
            su?: boolean;
          }) {
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
                this.remoteModuleSearchResults = _.map(
                  response.modules,
                  (m) => ({
                    ...m,
                    hash: m.latestHash,
                    hashCreatedAt: m.latestHashCreatedAt,
                  }),
                );
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

          async GET_REMOTE_MODULE_SPEC(id: ModuleId) {
            return new ApiRequest({
              method: "get",
              url: "/pkg/remote_module_spec",
              params: { id, ...visibility },
              onSuccess: (response) => {
                this.remoteModuleSpecsById[id] = response;
              },
            });
          },

          async INSTALL_REMOTE_MODULE(moduleId: ModuleId) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            this.installingModuleId = null;
            this.installingLoading = true;
            this.installingError = undefined;

            return new ApiRequest<{ id: string }>({
              method: "post",
              url: "/pkg/install_pkg",
              params: {
                id: moduleId,
                ...visibility,
              },
              onSuccess: (data) => {
                this.installingModuleId = data.id;
              },
              onFail: () => {
                this.installingModuleId = null;
                this.installingLoading = false;
              },
            });
          },

          async REJECT_REMOTE_MODULE(moduleId: ModuleId) {
            return new ApiRequest<{ success: true }>({
              method: "post",
              url: "/pkg/reject_pkg",
              params: { id: moduleId, ...visibility },
              onSuccess: (_response) => {
                // response is just success, so we have to reload the remote modules
                this.LOAD_LOCAL_MODULES();
                this.SEARCH_REMOTE_MODULES();
              },
            });
          },

          async PROMOTE_TO_BUILTIN(moduleId: ModuleId) {
            return new ApiRequest<{ success: true }>({
              method: "post",
              url: "/pkg/set_as_builtin",
              params: { id: moduleId, ...visibility },
              onSuccess: (_response) => {
                // response is just success, so we have to reload the remote modules
                this.SEARCH_REMOTE_MODULES();
              },
            });
          },

          async EXPORT_WORKSPACE() {
            this.exportingWorkspaceOperationId = null;
            this.exportingWorkspaceOperationError = undefined;
            this.exportingWorkspaceOperationRunning = true;

            return new ApiRequest<{ id: string }>({
              method: "post",
              url: "/pkg/export_workspace",
              params: { ...visibility },
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

          async EXPORT_MODULE(exportRequest: PkgExportRequest) {
            return new ApiRequest({
              method: "post",
              url: "/pkg/export_pkg",
              params: { ...exportRequest, ...visibility },
            });
          },
        },
        onActivated() {
          this.LOAD_LOCAL_MODULES();
          this.LIST_BUILTINS();

          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(this.$id, `workspace/${workspaceId}`, [
            {
              eventType: "ModuleImported",
              callback: () => {
                if (!this.installingModuleId) {
                  window.location.reload();
                }
              },
            },
            {
              eventType: "AsyncFinish",
              callback: async ({ id }: { id: string }) => {
                if (id === this.installingModuleId) {
                  this.installingError = undefined;
                  this.installingModuleId = null;
                  await this.LOAD_LOCAL_MODULES();
                  this.installingLoading = false;
                }
                if (id === this.exportingWorkspaceOperationId) {
                  this.exportingWorkspaceOperationRunning = false;
                }
              },
            },
            {
              eventType: "AsyncError",
              callback: async ({
                id,
                error,
              }: {
                id: string;
                error: string;
              }) => {
                if (id === this.installingModuleId) {
                  this.installingLoading = false;
                  this.installingError = error;
                  this.installingModuleId = null;
                }
                if (id === this.exportingWorkspaceOperationId) {
                  this.exportingWorkspaceOperationError = error;
                  this.exportingWorkspaceOperationRunning = false;
                }
              },
            },
          ]);

          return () => {
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
