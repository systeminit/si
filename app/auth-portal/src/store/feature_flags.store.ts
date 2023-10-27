import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import posthog from "posthog-js";

export const useFeatureFlagsStore = () => {
  return addStoreHooks(
    defineStore("feature-flags", {
      state: () => ({
        OSS_RELEASE: false,
        CREATE_WORKSPACES: false,
        EDIT_WORKSPACES: false,
        INVITE_USER: false,
      }),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          this.OSS_RELEASE = flags.includes("featureOssRelease");
          this.CREATE_WORKSPACES = flags.includes("create_workspaces");
          this.INVITE_USER = flags.includes("invite_user");
          // If you can create workspaces, editing workspaces will also be enabled.
          this.EDIT_WORKSPACES =
            flags.includes("edit_workspaces") || this.CREATE_WORKSPACES;
        });
      },
    }),
  )();
};
