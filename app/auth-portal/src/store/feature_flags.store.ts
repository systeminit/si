import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import posthog from "posthog-js";

export const useFeatureFlagsStore = () => {
  return addStoreHooks(
    defineStore("feature-flags", {
      state: () => ({
        INSTALL_PAGE: false,
        OSS_RELEASE: false,
        CREATE_WORKSPACES: false,
        EDIT_WORKSPACES: false,
      }),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          this.INSTALL_PAGE = flags.includes("install_page");
          this.OSS_RELEASE = flags.includes("featureOssRelease");
          this.CREATE_WORKSPACES = flags.includes("create_workspaces");
          // If you can create workspaces, editing workspaces will also be enabled.
          this.EDIT_WORKSPACES =
            flags.includes("edit_workspaces") || this.CREATE_WORKSPACES;
        });
      },
    }),
  )();
};
