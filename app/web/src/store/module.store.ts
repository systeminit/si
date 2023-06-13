import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { DiagramInputSocket, DiagramOutputSocket } from "@/api/sdf/dal/diagram";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "./change_sets.store";
import { useRouterStore } from "./router.store";
import { ModuleIndexApiRequest } from ".";

export type ModuleId = string;
export type ModuleSlug = string;
export type ModuleHash = string;

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
  latestHash: ModuleHash;
  latestHashCreatedAt: IsoDateString;
  ownerDisplayName: string;
  ownerUserId: string; // userid?
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
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/modules`, {
      state: () => ({
        localModulesByName: {} as Record<ModuleSlug, LocalModuleSummary>,
        localModuleDetailsByName: {} as Record<ModuleSlug, LocalModuleDetails>,

        remoteModuleSearchResults: [] as RemoteModuleSummary[],
        remoteModuleDetailsById: {} as Record<ModuleId, RemoteModuleDetails>,
      }),
      getters: {
        urlSelectedModuleSlug: () => {
          const route = useRouterStore().currentRoute;
          return route?.params?.moduleSlug as ModuleSlug | undefined;
        },
        localModules: (state) => _.values(state.localModulesByName),
        remoteModuleSummaryByName: (state) => {
          return _.keyBy(state.remoteModuleSearchResults, (m) => m.name);
        },
        remoteModuleDetailsByName: (state) => {
          return _.keyBy(
            _.values(state.remoteModuleDetailsById),
            (m) => m.name,
          );
        },

        selectedModuleLocalSummary(): LocalModuleSummary | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.localModulesByName[this.urlSelectedModuleSlug];
        },
        selectedModuleLocalDetails(): LocalModuleDetails | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.localModuleDetailsByName[this.urlSelectedModuleSlug];
        },
        selectedModuleRemoteDetails(): RemoteModuleDetails | undefined {
          if (!this.urlSelectedModuleSlug) return undefined;
          return this.remoteModuleDetailsByName[this.urlSelectedModuleSlug];
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

        async SEARCH_REMOTE_MODULES(nameQuery?: string) {
          return new ModuleIndexApiRequest<{ modules: RemoteModuleSummary[] }>({
            method: "get",
            url: "/modules",
            params: { name: nameQuery },
            onSuccess: (response) => {
              this.remoteModuleSearchResults = response.modules;
            },
          });
        },
        async GET_REMOTE_MODULE_DETAILS(id: ModuleId) {
          return new ModuleIndexApiRequest<RemoteModuleDetails>({
            method: "get",
            url: `/modules/${id}`,
            onSuccess: (response) => {
              this.remoteModuleDetailsById[response.id] = response;
            },
          });
        },

        async INSTALL_REMOTE_MODULE(moduleId: ModuleId) {
          return new ApiRequest<{ success: true }>({
            method: "post",
            url: "/pkg/install_pkg",
            params: { id: moduleId, ...visibility },
            onSuccess: (_response) => {
              // response is just success, so we have to reload local modules
              this.LOAD_LOCAL_MODULES();
            },
          });
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
      },
    }),
  )();
};
