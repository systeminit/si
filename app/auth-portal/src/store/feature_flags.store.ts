import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import { posthog } from "@/utils/posthog";

export const useFeatureFlagsStore = () => {
  return addStoreHooks(
    defineStore("feature-flags", {
      state: () => ({
        INSTALL_PAGE: false,
      }),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          this.INSTALL_PAGE = flags.includes("install_page");
        });
      },
    }),
  )();
};
