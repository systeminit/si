import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { parseISO } from "date-fns";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { DiagramInputSocket, DiagramOutputSocket } from "@/api/sdf/dal/diagram";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "./change_sets.store";
import { useRouterStore } from "./router.store";
import { ModuleIndexApiRequest } from ".";

export type ModuleId = string;
export type ModuleSlug = string;

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

export interface Package {
  name: string;
  version: string;
  description: string;
  createdAt: Date;
  createdBy: string;
  schemas: string[];
  funcs: PkgFuncView[];
  installed: boolean;
  hash: string;
}

export interface PkgGetResponse {
  name: string;
  version: string;
  description: string;
  createdAt: string;
  createdBy: string;
  schemas: string[];
  funcs: PkgFuncView[];
  installed: boolean;
  hash: string;
}

export interface PkgExportRequest {
  name: string;
  version: string;
  description?: string;
  schemaVariants: string[];
}

export interface PackageListItem {
  name: string;
  installed: boolean;
  hash: string;
}

export type Asset = {
  id: number;
  displayName: string;
};

export type ModuleSummary = {
  id: ModuleId;
  name: string;
  description: string;
};
export type ModuleDetails = ModuleSummary & { more: string };

export const useModuleStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: changeSetId ?? nilId(),
  };
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/package`, {
      state: () => ({
        packagesByName: {} as Record<ModuleId, Package>,
        packageListByName: {} as Record<ModuleId, PackageListItem>,

        modulesSearchResults: [] as ModuleSummary[],
        moduleDetailsById: {} as Record<ModuleId, ModuleDetails>,
      }),
      getters: {
        urlSelectedPackageSlug: () => {
          const route = useRouterStore().currentRoute;
          return route?.params?.packageSlug as ModuleSlug | undefined;
        },
        packageList: (state) => _.values(state.packageListByName),
        packagesBySlug: (state) =>
          _.keyBy(_.values(state.packageListByName), (p) => p.name),
        selectedPackageListItem(): PackageListItem | undefined {
          return this.packagesBySlug[this.urlSelectedPackageSlug ?? ""];
        },
        selectedPackage(): Package | undefined {
          return this.packagesByName[this.selectedPackageListItem?.name ?? ""];
        },
        installedPackages: (state) =>
          _.filter(state.packageListByName, (p) => p.installed),
        notInstalledPackages: (state) =>
          _.filter(state.packageListByName, (p) => !p.installed),

        modulesSearchResultsById: (state) =>
          _.keyBy(state.modulesSearchResults, (m) => m.id),
      },
      actions: {
        async GET_MODULE(pkg: PackageListItem) {
          return new ApiRequest<PkgGetResponse>({
            method: "get",
            url: "/pkg/get_pkg",
            params: { name: pkg.name, ...visibility },
            onSuccess: (response) => {
              this.packagesByName[pkg.name] = {
                ...response,
                createdAt: parseISO(response.createdAt),
              };
            },
          });
        },

        async INSTALL_MODULE(pkg: PackageListItem) {
          return new ApiRequest({
            method: "post",
            url: "/pkg/install_pkg",
            params: { name: pkg.name, ...visibility },
            onSuccess: (_response) => {
              const pkgItem = this.packagesByName[pkg.name];
              if (pkgItem) {
                pkgItem.installed = true;
                this.packagesByName[pkg.name] = pkgItem;
              }
              const pkgListItem = this.packageListByName[pkg.name];
              if (pkgListItem) {
                pkgListItem.installed = true;
                this.packageListByName[pkg.name] = pkgListItem;
              }
            },
          });
        },

        async LOAD_MODULES() {
          return new ApiRequest<{ pkgs: PackageListItem[] }>({
            url: "/pkg/list_pkgs",
            params: { ...visibility },
            onSuccess: (response) => {
              for (const pkg of response.pkgs) {
                this.packageListByName[pkg.name] = {
                  name: pkg.name,
                  installed: pkg.installed,
                  hash: pkg.hash,
                };
              }
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

        async SEARCH_MODULES(nameQuery?: string) {
          return new ModuleIndexApiRequest<{ modules: ModuleSummary[] }>({
            method: "get",
            url: "/modules",
            params: { name: nameQuery },
            onSuccess: (response) => {
              this.modulesSearchResults = response.modules;
            },
          });
        },
        async GET_MODULE_DETAILS(id: string) {
          return new ModuleIndexApiRequest<ModuleDetails>({
            method: "get",
            url: `/modules/${id}`,
            onSuccess: (response) => {
              this.moduleDetailsById[response.id] = response;
            },
          });
        },
      },
      onActivated() {
        this.LOAD_MODULES();
      },
    }),
  )();
};
