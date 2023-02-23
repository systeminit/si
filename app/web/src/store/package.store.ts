import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/store/lib/pinia_hooks_plugin";
import { IconNames, ICONS } from "@/ui-lib/icons/icon_set";
import { DiagramInputSocket, DiagramOutputSocket } from "@/api/sdf/dal/diagram";
import { ApiRequest } from "@/store/lib/pinia_api_tools";
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

export type Package = {
  id: PackageId; // TODO FUTURE - should probably have a namespace system for packages
  displayName: string;
  slug: PackageSlug; // TODO FUTURE - should probably have a namespace system for packages
  description?: string; // TODO - think about how this will be used, maybe two fields, one short one long? markdown?
  version: string; // TODO FUTURE - how do users select versions?
  schemaVariants: Array<SchemaVariant>;
  icon: IconNames;
  color: string;
  installed: boolean;
  createdAt: Date;
  createdBy: string;
  changelog: string;

  // TODO FUTURE - what other info would be useful here?
};

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
        packagesById: {} as Record<PackageId, Package>,
      }),
      getters: {
        urlSelectedPackageSlug: () => {
          const route = useRouterStore().currentRoute;
          return route?.params?.packageSlug as PackageSlug | undefined;
        },
        packages: (state) => _.values(state.packagesById),
        packagesBySlug: (state) =>
          _.keyBy(_.values(state.packagesById), (p) => p.slug),
        selectedPackage(): Package {
          return this.packagesBySlug[this.urlSelectedPackageSlug || ""];
        },
        installedPackages: (state) =>
          _.filter(state.packagesById, (p) => p.installed),
        notInstalledPackages: (state) =>
          _.filter(state.packagesById, (p) => !p.installed),
      },
      actions: {
        // MOCK DATA GENERATION
        generateMockColor() {
          return `#${_.sample([
            "FF0000",
            "FFFF00",
            "FF00FF",
            "00FFFF",
            "FFAA00",
            "AAFF00",
            "00FFAA",
            "00AAFF",
          ])}`;
        },
        generateMockPackage(id: string, name?: string, installed?: boolean) {
          return {
            id,
            displayName:
              name ??
              `test package ${Math.floor(Math.random() * 10000)}${
                Math.floor(Math.random() * 20) === 0
                  ? " omg has such a long name the name is so long you can't even believe how long it is!"
                  : ""
              }`,
            version: `${Math.floor(Math.random() * 9)}.${Math.floor(
              Math.random() * 9,
            )}`,
            schemaVariants: this.generateMockSchemaVariants(),
            icon: (_.sample(_.keys(ICONS)) || "logo-si") as IconNames,
            color: this.generateMockColor(),
            slug: `test${id}`,
            installed: installed ?? false,
            createdAt: new Date(
              new Date().getTime() - Math.random() * 10000000000,
            ),
            createdBy: "Fake McMock",
            changelog:
              _.sample([
                "changelog goes here",
                "testing changelog",
                "yeah this is fake",
              ]) || "changelog would go here",
          };
        },
        generateMockSchemaVariants() {
          const mockSchemaVariants = [] as SchemaVariant[];
          const amount = 2 + Math.floor(Math.random() * 30);

          for (let i = 0; i < amount; i++) {
            mockSchemaVariants.push({
              id: `${i}`,
              name: `test schema variant ${Math.floor(Math.random() * 10000)}`,
              schemaName: "whatever schema name",
              schemaId: `${i}`,
              color: this.generateMockColor(),
              inputSockets: [],
              outputSockets: [],
            });
          }

          return mockSchemaVariants;
        },

        async INSTALL_PACKAGE(pkg: Package) {
          return new ApiRequest({
            method: "post",
            url: "/pkg/install_pkg",
            params: { name: pkg.displayName, ...visibility },
            onSuccess: (response) => {
              this.packagesById[pkg.id].installed = true;
            },
          });
        },

        async LOAD_PACKAGES() {
          return new ApiRequest({
            url: "/pkg/list_pkgs",
            params: { ...visibility },
            onSuccess: (response) => {
              for (const [idx, pkg] of response.pkgs.entries()) {
                this.packagesById[idx + 1] = this.generateMockPackage(
                  idx + 1,
                  pkg.name,
                  pkg.installed,
                );
              }
            },
          });
        },
      },
      onActivated() {
        this.LOAD_PACKAGES();
      },
    }),
  )();
};
