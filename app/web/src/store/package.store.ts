import { defineStore } from "pinia";
import _ from "lodash";
import { parseISO } from "date-fns";
import { addStoreHooks, ApiRequest } from "@si/vue-lib";
import { DiagramInputSocket, DiagramOutputSocket } from "@/api/sdf/dal/diagram";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "./change_sets.store";
import { useRouterStore } from "./router.store";

export type PackageId = string;
export type PackageSlug = string;

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

export const usePackageStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: changeSetId ?? nilId(),
  };
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/package`, {
      state: () => ({
        packagesByName: {} as Record<PackageId, Package>,
        packageListByName: {} as Record<PackageId, PackageListItem>,
      }),
      getters: {
        urlSelectedPackageSlug: () => {
          const route = useRouterStore().currentRoute;
          return route?.params?.packageSlug as PackageSlug | undefined;
        },
        packageList: (state) => _.values(state.packageListByName),
        packagesBySlug: (state) =>
          _.keyBy(_.values(state.packageListByName), (p) => p.name),
        selectedPackageListItem(): PackageListItem {
          return this.packagesBySlug[this.urlSelectedPackageSlug ?? ""];
        },
        selectedPackage(): Package | undefined {
          return this.packagesByName[this.selectedPackageListItem?.name ?? ""];
        },
        installedPackages: (state) =>
          _.filter(state.packageListByName, (p) => p.installed),
        notInstalledPackages: (state) =>
          _.filter(state.packageListByName, (p) => !p.installed),
      },
      actions: {
        async GET_PACKAGE(pkg: PackageListItem) {
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

        async INSTALL_PACKAGE(pkg: PackageListItem) {
          return new ApiRequest({
            method: "post",
            url: "/pkg/install_pkg",
            params: { name: pkg.name, ...visibility },
            onSuccess: (_response) => {
              this.packagesByName[pkg.name].installed = true;
              this.packageListByName[pkg.name].installed = true;
            },
          });
        },

        async LOAD_PACKAGES() {
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

        async EXPORT_PACKAGE(exportRequest: PkgExportRequest) {
          return new ApiRequest({
            method: "post",
            url: "/pkg/export_pkg",
            params: { ...exportRequest, ...visibility },
          });
        },
      },
      onActivated() {
        this.LOAD_PACKAGES();
      },
    }),
  )();
};
