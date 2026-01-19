import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import posthog from "posthog-js";

export const useFeatureFlagsStore = () => {
  return addStoreHooks(
    undefined,
    undefined,
    defineStore("feature-flags", {
      state: () => ({
        ADMIN_PAGE: false,
        APPROVALS_OPT_IN_OUT: false,
        SECURE_BEARER_TOKENS: false,
      }),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          this.ADMIN_PAGE = flags.includes("auth_portal_admin_page");
          this.APPROVALS_OPT_IN_OUT = flags.includes("approvals-opt-in-out");
          this.SECURE_BEARER_TOKENS = flags.includes("secure-bearer-tokens");
        });
      },
    }),
  )();
};
