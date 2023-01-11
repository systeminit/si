import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { useChangeSetsStore } from "./change_sets.store";

export type PackageId = string;

export type Package = {
  name: string;
  version: string;
  schemaVariants: Array<string>;
};

export const usePackageStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/package`, {
      state: () => ({
        packages: {} as Record<PackageId, Package>,
      }),
      getters: {},
      actions: {},
      onActivated() {
        this.packages = {
          0: { name: "test package 0", version: "1.0", schemaVariants: [] },
          1: { name: "test package 1", version: "13.12", schemaVariants: [] },
          2: { name: "test package 2", version: "4.20", schemaVariants: [] },
          3: { name: "test package 3", version: "7.7", schemaVariants: [] },
          4: { name: "test package 4", version: "6.66", schemaVariants: [] },
        };
      },
    }),
  )();
};
