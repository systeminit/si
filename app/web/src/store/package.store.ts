import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { IconNames } from "@/ui-lib/icons/icon_set";
import { DiagramSchemaVariant } from "@/api/sdf/dal/diagram";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { useChangeSetsStore } from "./change_sets.store";

export type PackageId = string;

export type Package = {
  id: PackageId;
  displayName: string;
  slug: string;
  description?: string; // TODO - think about how this will be used, maybe two fields, one short one long? markdown?
  version: string; // FUTURE - how do users select versions?
  schemaVariants: Array<DiagramSchemaVariant>; // TODO - use my own type here
  icon: IconNames;
  color?: string; // TODO - use this, not optional
  // TODO - what else do packages have? talk to fletcher
  // namespace/org slug perhaps? created at? created by? etc. think about what's in a package json in NPM
};

export const usePackageStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/package`, {
      state: () => ({
        packagesById: {} as Record<PackageId, Package>,
        selectedPackageId: null as PackageId | null,
      }),
      getters: {
        packages: (state) => _.values(state.packagesById),
        selectedPackage(): Package {
          return this.packagesById[this.selectedPackageId || 0];
        },
      },
      actions: {
        setSelectedPackageId(selection: PackageId | null) {
          if (!selection) {
            this.selectedPackageId = null;
          } else {
            if (this.packagesById[selection]) {
              this.selectedPackageId = selection;
            }
          }
        },
        setSelectedPackageBySlug(selection: string | null) {
          if (!selection) {
            this.selectedPackageId = null;
          } else {
            const pkg = _.find(this.packages, (p) => p.slug === selection);
            if (pkg) {
              this.selectedPackageId = pkg.id;
            }
          }
        },
        async LOAD_PACKAGES() {
          return new ApiRequest({
            url: "/session/restore_authentication", // TODO - replace with real API request
            onSuccess: () => {
              this.packagesById = {
                666: {
                  id: "666",
                  displayName: "test package 0",
                  version: "1.0",
                  schemaVariants: [
                    {
                      id: "1",
                      name: "test schema variant 1",
                      schemaName: "whatever schema name",
                      schemaId: "1",
                      color: 1,
                      inputSockets: [],
                      outputSockets: [],
                    },
                  ],
                  icon: "cat",
                  slug: "cat",
                },
                123: {
                  id: "123",
                  displayName: "test package 1",
                  version: "13.12",
                  schemaVariants: [],
                  icon: "bolt",
                  slug: "bolt",
                },
                777: {
                  id: "777",
                  displayName: "test package 2",
                  version: "4.20",
                  schemaVariants: [],
                  icon: "alert-circle",
                  slug: "alert",
                },
                "string ids": {
                  id: "string ids",
                  displayName: "test package 3",
                  version: "7.7",
                  schemaVariants: [],
                  icon: "sun",
                  slug: "sun",
                },
                test: {
                  id: "test",
                  displayName: "test package 4",
                  version: "6.66",
                  schemaVariants: [],
                  icon: "logo-si",
                  slug: "si",
                },
              };
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
